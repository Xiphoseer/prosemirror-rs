use super::{MarkdownMark, MarkdownNode, MD};
use crate::model::{AttrNode, Block, Fragment, Leaf, Node};
use displaydoc::Display;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, InlineStr, LinkType, Tag};
use pulldown_cmark_to_cmark::cmark;
use thiserror::Error;

/// Possible error when generating markdown
#[derive(Debug, Clone, PartialEq, Eq, Display, Error)]
pub struct ToMarkdownError {
    /// The inner error
    inner: std::fmt::Error,
}

impl From<std::fmt::Error> for ToMarkdownError {
    fn from(e: std::fmt::Error) -> ToMarkdownError {
        Self { inner: e }
    }
}

/// Turn a markdown document into a string
pub fn to_markdown(doc: &MarkdownNode) -> Result<String, ToMarkdownError> {
    let mut buf = String::with_capacity(doc.node_size() + 128);
    let events = MarkdownSerializer::new(doc);
    cmark(events, &mut buf, None)?;
    Ok(buf)
}

struct MarkdownSerializer<'a> {
    inner: Vec<(&'a MarkdownNode, usize)>,
    marks: Vec<&'a MarkdownMark>,
    stack: Vec<Event<'a>>,
}

impl<'a> MarkdownSerializer<'a> {
    fn new(doc: &'a MarkdownNode) -> Self {
        Self {
            inner: vec![(doc, 0)],
            marks: vec![],
            stack: vec![],
        }
    }
}

fn mark_tag(mark: &MarkdownMark) -> Tag {
    match mark {
        MarkdownMark::Strong => Tag::Strong,
        MarkdownMark::Em => Tag::Emphasis,
        MarkdownMark::Code => unimplemented!("Should not be pushed on the mark stack"),
        MarkdownMark::Link { attrs } => Tag::Link(
            LinkType::Inline,
            CowStr::Borrowed(attrs.href.as_str()),
            CowStr::Borrowed(attrs.title.as_str()),
        ),
    }
}

impl<'a> MarkdownSerializer<'a> {
    fn process_content(
        &mut self,
        index: usize,
        content: &'a Fragment<MD>,
        node: &'a MarkdownNode,
    ) -> bool {
        if let Some(child) = content.maybe_child(index) {
            self.inner.push((node, index + 1));
            self.inner.push((child, 0));
            false
        } else {
            true
        }
    }

    fn process_attr_node<A, F>(
        &mut self,
        index: usize,
        content: &'a Fragment<MD>,
        attrs: &'a A,
        node: &'a MarkdownNode,
        map: F,
    ) -> Option<Event<'a>>
    where
        F: FnOnce(&'a A) -> Tag<'a>,
    {
        if index == 0 {
            if let Some(mark) = self.marks.pop() {
                self.inner.push((node, 0));
                return Some(Event::End(mark_tag(mark)));
            }
        }
        let last = self.process_content(index, content, node);
        if index == 0 {
            if last {
                // close the tag next
                self.inner.push((node, index + 1));
            }
            Some(Event::Start(map(attrs)))
        } else if last {
            if let Some(mark) = self.marks.pop() {
                self.inner.push((node, index));
                return Some(Event::End(mark_tag(mark)));
            }
            let tag = map(attrs);
            if matches!(&tag, Tag::CodeBlock(..)) {
                self.stack.push(Event::End(tag));
                Some(Event::Text(CowStr::Inlined(InlineStr::from('\n'))))
            } else {
                Some(Event::End(tag))
            }
        } else {
            self.next()
        }
    }
}

impl<'a> Iterator for MarkdownSerializer<'a> {
    type Item = Event<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ev) = self.stack.pop() {
            return Some(ev);
        }

        if let Some((node, index)) = self.inner.pop() {
            match node {
                MarkdownNode::Doc(Block { content }) => {
                    self.process_content(index, content, node);
                    self.next()
                }
                MarkdownNode::Heading(AttrNode { attrs, content }) => {
                    self.process_attr_node(index, content, attrs, node, |attrs| {
                        Tag::Heading(attrs.level.into())
                    })
                }
                MarkdownNode::CodeBlock(AttrNode { attrs, content }) => {
                    self.process_attr_node(index, content, attrs, node, |attrs| {
                        Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(&attrs.params)))
                    })
                }
                MarkdownNode::Text(text_node) => {
                    if let Some(last) = self.marks.last().map(|n| *n) {
                        if !text_node.marks.contains(last) {
                            self.inner.push((node, index));
                            self.marks.pop();
                            return Some(Event::End(mark_tag(last)));
                        }
                    }
                    let mut is_code = false;
                    for mark in &text_node.marks {
                        if *mark == MarkdownMark::Code {
                            is_code = true;
                        } else if !self.marks.contains(&mark) {
                            self.inner.push((node, index));
                            self.marks.push(mark);
                            return Some(Event::Start(mark_tag(mark)));
                        }
                    }
                    if is_code {
                        Some(Event::Code(CowStr::Borrowed(text_node.text.as_str())))
                    } else {
                        Some(Event::Text(CowStr::Borrowed(text_node.text.as_str())))
                    }
                }
                MarkdownNode::Blockquote(Block { content }) => {
                    self.process_attr_node(index, content, &(), node, |()| Tag::BlockQuote)
                }
                MarkdownNode::Paragraph(Block { content }) => {
                    self.process_attr_node(index, content, &(), node, |()| Tag::Paragraph)
                }
                MarkdownNode::BulletList(AttrNode { attrs, content }) => {
                    self.process_attr_node(index, content, attrs, node, |_| Tag::List(None))
                }
                MarkdownNode::OrderedList(AttrNode { attrs, content }) => {
                    self.process_attr_node(index, content, attrs, node, |_| {
                        Tag::List(Some(attrs.order as u64))
                    })
                }
                MarkdownNode::ListItem(Block { content }) => {
                    self.process_attr_node(index, content, &(), node, |()| Tag::Item)
                }
                MarkdownNode::HorizontalRule => Some(Event::Rule),
                MarkdownNode::HardBreak => {
                    // todo: inline marks
                    Some(Event::HardBreak)
                }
                MarkdownNode::Image(Leaf { attrs }) => {
                    self.process_attr_node(index, Fragment::EMPTY_REF, &(), node, |()| {
                        Tag::Image(
                            LinkType::Inline,
                            CowStr::Borrowed(attrs.src.as_str()),
                            CowStr::Borrowed(attrs.src.as_str()),
                        )
                    })
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use super::to_markdown;
    use crate::markdown::{
        helper::{blockquote, code_block, doc, h1, h2, node, p, strong},
        MarkdownNode,
    };

    fn initial_doc() -> MarkdownNode {
        doc(vec![
            h1((
                "Padington",
            )),
            code_block("", (
                "fn foo(a: u32) -> u32 {\n  2 * a\n}",
            )),
            h2((
                "Lorem Ipsum",
            )),
            blockquote((
                p(vec![
                    node("Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. "),
                    strong("At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."),
                    node(" Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet."),
                ]),
            ))
        ])
    }

    #[test]
    fn test() {
        let node = initial_doc();
        let res = "\
        # Padington\n\
        \n\
        ````\n\
        fn foo(a: u32) -> u32 {\n  2 * a\n}\n\
        ````\n\
        \n\
        ## Lorem Ipsum\n\
        \n > \n > Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. **At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.** Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.\
        ".to_string();

        let save = to_markdown(&node);
        let line = "\n---------------------------------------\n";
        assert_eq!(
            save.as_ref(),
            Ok(&res),
            "\ngenerated:{}{}{}\n\nexpected:{}{}{}\n",
            line,
            save.as_ref().unwrap(),
            line,
            line,
            &res,
            line,
        );
    }
}
