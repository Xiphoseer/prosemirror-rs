use super::{
    BulletListAttrs, CodeBlockAttrs, HeadingAttrs, ImageAttrs, LinkAttrs, MarkdownMark,
    MarkdownNode, OrderedListAttrs, MD,
};
use crate::model::{AttrNode, Block, Fragment, Leaf, MarkSet, Text, TextNode};
use displaydoc::Display;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};
use std::{
    convert::{TryFrom, TryInto},
    num::TryFromIntError,
};
use thiserror::Error;

/// Errors that can occur when reading a markdown file
#[derive(Debug, PartialEq, Display, Error)]
pub enum FromMarkdownError {
    /// Heading level too deep
    LevelMismatch(#[from] TryFromIntError),
    /// Not supported: `{0}`
    NotSupported(&'static str),
    /// The stack was empty
    StackEmpty,
    /// Event mismatch
    MisplacedEndTag(&'static str, Attrs),
    /// No children allowed in {0:?}
    NoChildrenAllowed(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attrs {
    Doc,
    Paragraph,
    Heading(HeadingAttrs),
    Blockquote,
    CodeBlock(CodeBlockAttrs),
    OrderedList(OrderedListAttrs),
    BulletList(BulletListAttrs),
    ListItem,
    Image(ImageAttrs),
}

/// Creates a MarkdownNode::Doc from a text
pub fn from_markdown(text: &str) -> Result<MarkdownNode, FromMarkdownError> {
    let parser = Parser::new(text);
    let mut d = MarkdownDeserializer::default();
    d.deserialize(parser)
}

#[derive(Default)]
pub struct MarkdownDeserializer {
    stack: Vec<(Vec<MarkdownNode>, Attrs)>,
    mark_set: MarkSet<MD>,
}

impl MarkdownDeserializer {
    /*#[must_use]
    fn push_text(&mut self) -> Result<(), FromMarkdownError> {
        let last = self.stack.last_mut().ok_or(FromMarkdownError::StackEmpty)?;
        if !self.text.is_empty() {
            last.0.push(MarkdownNode::Text(TextNode {
                marks: self.mark_set.clone(),
                text: Text::from(std::mem::take(&mut self.text)),
            }));
        }
        Ok(())
    }*/

    fn push_stack(&mut self, attrs: Attrs) {
        self.stack.push((Vec::new(), attrs));
    }

    fn pop_stack(&mut self) -> Result<(Vec<MarkdownNode>, Attrs), FromMarkdownError> {
        let popped = self.stack.pop().ok_or(FromMarkdownError::StackEmpty)?;
        Ok(popped)
    }

    fn add_content(&mut self, node: MarkdownNode) -> Result<(), FromMarkdownError> {
        let last = self.stack.last_mut().ok_or(FromMarkdownError::StackEmpty)?;
        last.0.push(node);
        Ok(())
    }

    fn deserialize(&mut self, parser: Parser) -> Result<MarkdownNode, FromMarkdownError> {
        self.push_stack(Attrs::Doc);
        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Paragraph => {
                        self.stack.push((Vec::new(), Attrs::Paragraph));
                    }
                    Tag::Heading(l) => {
                        let level = u8::try_from(l)?;
                        self.stack
                            .push((Vec::new(), Attrs::Heading(HeadingAttrs { level })));
                    }
                    Tag::BlockQuote => {
                        self.stack.push((Vec::new(), Attrs::Blockquote));
                    }
                    Tag::CodeBlock(kind) => {
                        let params = if let CodeBlockKind::Fenced(params) = kind {
                            params.to_string()
                        } else {
                            String::new()
                        };
                        self.stack
                            .push((Vec::new(), Attrs::CodeBlock(CodeBlockAttrs { params })));
                    }
                    Tag::List(ord) => {
                        if let Some(order) = ord {
                            self.stack.push((
                                Vec::new(),
                                Attrs::OrderedList(OrderedListAttrs {
                                    order: order.try_into()?, // TODO: other error
                                    tight: false,
                                }),
                            ))
                        } else {
                            self.stack.push((
                                Vec::new(),
                                Attrs::BulletList(BulletListAttrs { tight: false }),
                            ));
                        }
                    }
                    Tag::Item => {
                        self.stack.push((Vec::new(), Attrs::ListItem));
                    }
                    Tag::FootnoteDefinition(_) => {
                        return Err(FromMarkdownError::NotSupported("FootnoteDefinition"));
                    }
                    Tag::Table(_) => {
                        return Err(FromMarkdownError::NotSupported("Table"));
                    }
                    Tag::TableHead => {
                        return Err(FromMarkdownError::NotSupported("TableHead"));
                    }
                    Tag::TableRow => {
                        return Err(FromMarkdownError::NotSupported("TableRow"));
                    }
                    Tag::TableCell => {
                        return Err(FromMarkdownError::NotSupported("TableCell"));
                    }
                    Tag::Emphasis => {
                        self.mark_set.add(&MarkdownMark::Em);
                    }
                    Tag::Strong => {
                        self.mark_set.add(&MarkdownMark::Strong);
                    }
                    Tag::Strikethrough => {
                        return Err(FromMarkdownError::NotSupported("Strikethrough"));
                    }
                    Tag::Link(_, href, title) => {
                        self.mark_set.add(&MarkdownMark::Link {
                            attrs: LinkAttrs {
                                href: href.to_string(),
                                title: title.to_string(),
                            },
                        });
                    }
                    Tag::Image(_, src, title) => {
                        self.push_stack(Attrs::Image(ImageAttrs {
                            src: src.to_string(),
                            alt: title.to_string(),
                            title: title.to_string(),
                        }));
                    }
                },
                Event::End(tag) => match tag {
                    Tag::Paragraph => {
                        let (content, attrs) = self.pop_stack()?;
                        if matches!(attrs, Attrs::Paragraph) {
                            let p = MarkdownNode::Paragraph(Block {
                                content: Fragment::from(content),
                            });
                            self.add_content(p)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag("Paragraph", attrs));
                        }
                    }
                    Tag::Heading(_) => {
                        let (content, attrs) = self.pop_stack()?;
                        if let Attrs::Heading(attrs) = attrs {
                            let h = MarkdownNode::Heading(AttrNode {
                                attrs,
                                content: Fragment::from(content),
                            });
                            self.add_content(h)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag("Heading", attrs));
                        }
                    }
                    Tag::BlockQuote => {
                        let (content, attrs) = self.pop_stack()?;
                        if let Attrs::Blockquote = attrs {
                            let b = MarkdownNode::Blockquote(Block {
                                content: Fragment::from(content),
                            });
                            self.add_content(b)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag("BlockQuote", attrs));
                        }
                    }
                    Tag::CodeBlock(_) => {
                        let (mut content, attrs) = self.pop_stack()?;
                        if let Attrs::CodeBlock(attrs) = attrs {
                            if let Some(MarkdownNode::Text(t)) = content.last_mut() {
                                t.text.remove_last_newline();
                            }
                            let cb = MarkdownNode::CodeBlock(AttrNode {
                                attrs,
                                content: Fragment::from(content),
                            });
                            self.add_content(cb)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag("CodeBlock", attrs));
                        }
                    }
                    Tag::List(_) => {
                        let (content, attrs) = self.pop_stack()?;
                        match attrs {
                            Attrs::BulletList(attrs) => {
                                let l = MarkdownNode::BulletList(AttrNode {
                                    attrs,
                                    content: Fragment::from(content),
                                });
                                self.add_content(l)?;
                            }
                            Attrs::OrderedList(attrs) => {
                                let l = MarkdownNode::OrderedList(AttrNode {
                                    attrs,
                                    content: Fragment::from(content),
                                });
                                self.add_content(l)?;
                            }
                            _ => {
                                return Err(FromMarkdownError::MisplacedEndTag("List", attrs));
                            }
                        }
                    }
                    Tag::Item => {
                        let (content, attrs) = self.pop_stack()?;
                        if let Attrs::ListItem = attrs {
                            let cb = MarkdownNode::ListItem(Block {
                                content: Fragment::from(content),
                            });
                            self.add_content(cb)?;
                        }
                    }
                    Tag::FootnoteDefinition(_) => {
                        return Err(FromMarkdownError::NotSupported("FootnoteDefinition"));
                    }
                    Tag::Table(_) => {
                        return Err(FromMarkdownError::NotSupported("Table"));
                    }
                    Tag::TableHead => {
                        return Err(FromMarkdownError::NotSupported("TableHead"));
                    }
                    Tag::TableRow => {
                        return Err(FromMarkdownError::NotSupported("TableRow"));
                    }
                    Tag::TableCell => {
                        return Err(FromMarkdownError::NotSupported("TableCell"));
                    }
                    Tag::Emphasis => {
                        self.mark_set.remove(&MarkdownMark::Em);
                    }
                    Tag::Strong => {
                        self.mark_set.remove(&MarkdownMark::Strong);
                    }
                    Tag::Strikethrough => {
                        return Err(FromMarkdownError::NotSupported("Strikethrough"));
                    }
                    Tag::Link(_, href, title) => self.mark_set.remove(&MarkdownMark::Link {
                        attrs: LinkAttrs {
                            href: href.to_string(),
                            title: title.to_string(),
                        },
                    }),
                    Tag::Image(_, _, _) => {
                        let (content, attrs) = self.pop_stack()?;
                        if let Attrs::Image(attrs) = attrs {
                            if content.len() > 0 {
                                return Err(FromMarkdownError::NoChildrenAllowed("Image"));
                            }
                            let cb = MarkdownNode::Image(Leaf { attrs });
                            self.add_content(cb)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag("Image", attrs));
                        }
                    }
                },
                Event::Text(text) => {
                    self.add_content(MarkdownNode::Text(TextNode {
                        text: Text::from(text.to_string()),
                        marks: self.mark_set.clone(),
                    }))?;
                }
                Event::Code(text) => {
                    let mut marks = self.mark_set.clone();
                    marks.add(&MarkdownMark::Code);
                    self.add_content(MarkdownNode::Text(TextNode {
                        text: Text::from(text.to_string()),
                        marks,
                    }))?;
                }
                Event::Html(_) => {
                    return Err(FromMarkdownError::NotSupported("Html"));
                }
                Event::FootnoteReference(_) => {
                    return Err(FromMarkdownError::NotSupported("FootnoteReference"));
                }
                Event::SoftBreak => {
                    return Err(FromMarkdownError::NotSupported("SoftBreak"));
                }
                Event::HardBreak => {
                    self.add_content(MarkdownNode::HardBreak)?;
                }
                Event::Rule => {
                    self.add_content(MarkdownNode::HorizontalRule)?;
                }
                Event::TaskListMarker(_) => {
                    return Err(FromMarkdownError::NotSupported("TaskListMarker"));
                }
            }
        }
        let (content, attrs) = self.pop_stack()?;
        if let Attrs::Doc = attrs {
            Ok(MarkdownNode::Doc(Block {
                content: Fragment::from(content),
            }))
        } else {
            Err(FromMarkdownError::MisplacedEndTag("Doc", attrs))
        }
    }
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::{CowStr, Event, Parser, Tag};

    #[test]
    fn test_alerts() {
        let test_string = "\
        ### Alert Area\n\
        \n\
        :::success\n\
        Yes :tada:\n\
        :::\n\
        ";

        let p = Parser::new(test_string);
        let v: Vec<Event> = p.collect();
        assert_eq!(
            v,
            vec![
                Event::Start(Tag::Heading(3)),
                Event::Text(CowStr::Borrowed("Alert Area")),
                Event::End(Tag::Heading(3)),
                Event::Start(Tag::Paragraph),
                Event::Text(CowStr::Borrowed(":::success")),
                Event::SoftBreak,
                Event::Text(CowStr::Borrowed("Yes :tada:")),
                Event::SoftBreak,
                Event::Text(CowStr::Borrowed(":::")),
                Event::End(Tag::Paragraph),
            ]
        );
    }
}
