//! # The markdown schema
//!
//! This module is derived from the `prosemirror-markdown` schema and the
//! the general JSON serialization of nodes.
mod attrs;
pub mod helper;

use crate::model::{Block, Fragment, Mark, MarkSet, Node, Schema, Text};
pub use attrs::{
    BulletListAttrs, CodeBlockAttrs, HeadingAttrs, ImageAttrs, LinkAttrs, OrderedListAttrs,
};
use serde::{Deserialize, Serialize};

/// The markdown schema type
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownSchema;

impl Schema for MarkdownSchema {
    type Node = MarkdownNode;
    type Mark = MarkdownMark;
}

/// The node type for the markdown schema
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MarkdownNode {
    /// The document root
    Doc(Block<MarkdownSchema>),
    /// A heading, e.g. `<h1>`
    Heading {
        /// Attributes
        attrs: HeadingAttrs,
        /// The content.
        #[serde(default)]
        content: Fragment<MarkdownSchema>,
    },
    /// A code block
    CodeBlock {
        /// Attributes
        attrs: CodeBlockAttrs,
        /// The content.
        #[serde(default)]
        content: Fragment<MarkdownSchema>,
    },
    /// A text node
    Text {
        // todo: replace with typemap
        /// Marks on this node
        #[serde(default)]
        marks: MarkSet<MarkdownSchema>,
        /// The actual text
        text: Text,
    },
    /// A blockquote
    Blockquote(Block<MarkdownSchema>),
    /// A paragraph
    Paragraph(Block<MarkdownSchema>),
    /// A bullet list
    BulletList {
        /// The content.
        #[serde(default)]
        content: Fragment<MarkdownSchema>,
        /// Attributes.
        attrs: BulletListAttrs,
    },
    /// An ordered list
    OrderedList {
        /// The content.
        #[serde(default)]
        content: Fragment<MarkdownSchema>,
        /// Attributes
        attrs: OrderedListAttrs,
    },
    /// A list item
    ListItem {
        /// The content.
        #[serde(default)]
        content: Fragment<MarkdownSchema>,
    },
    /// A horizontal line `<hr>`
    HorizontalRule,
    /// A hard break `<br>`
    HardBreak,
    /// An image `<img>`
    Image {
        /// Attributes
        attrs: ImageAttrs,
    },
}

impl Node<MarkdownSchema> for MarkdownNode {
    fn text_node(&self) -> Option<(&Text, &MarkSet<MarkdownSchema>)> {
        if let Self::Text { text, marks } = self {
            Some((text, marks))
        } else {
            None
        }
    }

    fn new_text_node(text: Text, marks: MarkSet<MarkdownSchema>) -> Self {
        Self::Text { text, marks }
    }

    fn is_block(&self) -> bool {
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

    fn text<A: Into<String>>(text: A) -> Self {
        Self::Text {
            text: Text::from(text.into()),
            marks: MarkSet::<MarkdownSchema>::new(),
        }
    }

    fn content(&self) -> Option<&Fragment<MarkdownSchema>> {
        match self {
            Self::Doc(doc) => Some(&doc.content),
            Self::Heading { content, .. } => Some(content),
            Self::CodeBlock { content, .. } => Some(content),
            Self::Text { .. } => None,
            Self::Blockquote(Block { content }) => Some(content),
            Self::Paragraph(Block { content }) => Some(content),
            Self::BulletList { content, .. } => Some(content),
            Self::OrderedList { content, .. } => Some(content),
            Self::ListItem { content } => Some(content),
            Self::HorizontalRule => None,
            Self::HardBreak => None,
            Self::Image { .. } => None,
        }
    }

    fn copy<F>(&self, map: F) -> Self
    where
        F: FnOnce(&Fragment<MarkdownSchema>) -> Fragment<MarkdownSchema>,
    {
        match self {
            Self::Doc(Block { content }) => Self::Doc(Block {
                content: map(content),
            }),
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
            Self::Blockquote(Block { content }) => Self::Blockquote(Block {
                content: map(content),
            }),
            Self::Paragraph(Block { content }) => Self::Paragraph(Block {
                content: map(content),
            }),
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
}

impl From<&str> for MarkdownNode {
    fn from(text: &str) -> Self {
        Self::text(text)
    }
}

/// The marks that can be on some span
#[derive(Debug, Hash, Eq, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MarkdownMark {
    /// bold
    Strong,
    /// italics
    Em,
    /// monospace
    Code,
    /// hyper-linked
    Link {
        /// The attributes
        attrs: LinkAttrs,
    },
}

impl Mark for MarkdownMark {}
