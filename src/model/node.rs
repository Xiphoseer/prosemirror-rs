use super::{de, util, Fragment, MarkSet};
use serde::{Deserialize, Serialize, Serializer};
use std::borrow::Cow;
use std::ops::RangeBounds;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HeadingAttrs {
    pub level: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CodeBlockAttrs {
    pub params: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BulletListAttrs {
    tight: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OrderedListAttrs {
    pub order: usize,
    pub tight: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ImageAttrs {
    pub src: String,
    #[serde(default, deserialize_with = "de::deserialize_or_default")]
    pub alt: String,
    #[serde(default, deserialize_with = "de::deserialize_or_default")]
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(from = "String")]
pub struct Text {
    len_utf16: usize,
    content: String,
}

impl Text {
    pub fn as_str(&self) -> &str {
        &self.content
    }

    pub fn len_utf16(&self) -> usize {
        self.len_utf16
    }
}

impl From<String> for Text {
    fn from(src: String) -> Text {
        Text {
            len_utf16: src.encode_utf16().count(),
            content: src,
        }
    }
}

impl Serialize for Text {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.content.serialize(serializer)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Node {
    Doc {
        #[serde(default)]
        content: Fragment,
    },
    Heading {
        attrs: HeadingAttrs,
        #[serde(default)]
        content: Fragment,
    },
    CodeBlock {
        attrs: CodeBlockAttrs,
        #[serde(default)]
        content: Fragment,
    },
    Text {
        // todo: replace with typemap
        #[serde(default)]
        marks: MarkSet,
        text: Text,
    },
    Blockquote {
        #[serde(default)]
        content: Fragment,
    },
    Paragraph {
        #[serde(default)]
        content: Fragment,
    },
    BulletList {
        #[serde(default)]
        content: Fragment,
        attrs: BulletListAttrs,
    },
    OrderedList {
        #[serde(default)]
        content: Fragment,
        attrs: OrderedListAttrs,
    },
    ListItem {
        #[serde(default)]
        content: Fragment,
    },
    HorizontalRule,
    HardBreak,
    Image {
        attrs: ImageAttrs,
    },
}

impl From<&str> for Node {
    fn from(text: &str) -> Node {
        Self::text(text)
    }
}

impl Node {
    pub fn cut<R: RangeBounds<usize>>(&self, range: R) -> Cow<Node> {
        let from = util::from(&range);

        if let Node::Text { text, marks } = self {
            let len = text.len_utf16;
            let to = util::to(&range, len);

            if from == 0 && to == len {
                return Cow::Borrowed(self);
            }
            let (_, rest) = util::split_at_utf16(&text.content, from);
            let (rest, _) = util::split_at_utf16(rest, to - from);

            Cow::Owned(Node::Text {
                text: Text::from(rest.to_owned()),
                marks: marks.clone(),
            })
        } else {
            let content_size = self.content_size();
            let to = util::to(&range, content_size);

            if from == 0 && to == content_size {
                Cow::Borrowed(self)
            } else {
                Cow::Owned(self.copy(|c| c.cut(from..to)))
            }
        }
    }

    pub fn copy<F>(&self, map: F) -> Node
    where
        F: FnOnce(&Fragment) -> Fragment,
    {
        match self {
            Self::Doc { content } => Self::Doc {
                content: map(content),
            },
            Self::Heading { attrs, content } => Self::Heading {
                attrs: attrs.clone(),
                content: map(content),
            },
            Self::CodeBlock { attrs, content } => Self::CodeBlock {
                attrs: attrs.clone(),
                content: map(content),
            },
            Self::Text { text, marks } => Self::Text {
                text: text.clone(),
                marks: marks.clone(),
            },
            Self::Blockquote { content } => Self::Blockquote {
                content: map(content),
            },
            Self::Paragraph { content } => Self::Paragraph {
                content: map(content),
            },
            Self::BulletList { attrs, content } => Self::BulletList {
                attrs: attrs.clone(),
                content: map(content),
            },
            Self::OrderedList { attrs, content } => Self::OrderedList {
                attrs: attrs.clone(),
                content: map(content),
            },
            Self::ListItem { content } => Self::ListItem {
                content: map(content),
            },
            Self::HorizontalRule => Self::HorizontalRule,
            Self::HardBreak => Self::HardBreak,
            Self::Image { attrs } => Self::Image {
                attrs: attrs.clone(),
            },
        }
    }

    pub fn text_content(&self) -> String {
        if let Node::Text { text, .. } = self {
            text.content.clone()
        } else {
            let mut buf = String::new();
            if let Some(c) = self.content() {
                c.text_between(&mut buf, true, 0, c.size(), Some(""), None);
            }
            buf
        }
    }

    pub fn content_size(&self) -> usize {
        self.content().map(Fragment::size).unwrap_or(0)
    }

    pub fn text<A: Into<String>>(text: A) -> Self {
        Node::Text {
            text: Text::from(text.into()),
            marks: MarkSet::new(),
        }
    }

    pub fn content(&self) -> Option<&Fragment> {
        match self {
            Self::Doc { content } => Some(content),
            Self::Heading { content, .. } => Some(content),
            Self::CodeBlock { content, .. } => Some(content),
            Self::Text { .. } => None,
            Self::Blockquote { content } => Some(content),
            Self::Paragraph { content } => Some(content),
            Self::BulletList { content, .. } => Some(content),
            Self::OrderedList { content, .. } => Some(content),
            Self::ListItem { content } => Some(content),
            Self::HorizontalRule => None,
            Self::HardBreak => None,
            Self::Image { .. } => None,
        }
    }

    pub fn child(&self, index: usize) -> Option<&Node> {
        self.content().and_then(|c| c.child(index))
    }

    pub fn child_count(&self) -> usize {
        self.content().map_or(0, Fragment::count)
    }

    pub fn is_leaf(&self) -> bool {
        self.content().is_none()
    }

    pub fn is_block(&self) -> bool {
        match self {
            Self::Doc { .. } => true,
            Self::Paragraph { .. } => true,
            Self::Blockquote { .. } => true,
            Self::HorizontalRule => true,
            Self::Heading { .. } => true,
            Self::CodeBlock { .. } => true,
            Self::OrderedList { .. } => true,
            Self::BulletList { .. } => true,
            Self::ListItem { .. } => true,
            Self::Text { .. } => false,
            Self::Image { .. } => false,
            Self::HardBreak => false,
        }
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text {..})
    }

    pub fn node_size(&self) -> usize {
        match self.content() {
            Some(c) => c.size() + 2,
            None => {
                if let Self::Text { text, .. } = self {
                    text.len_utf16
                } else {
                    1
                }
            }
        }
    }
}
