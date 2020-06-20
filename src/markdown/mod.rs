//! # The markdown schema
//!
//! This module is derived from the `prosemirror-markdown` schema and the
//! the general JSON serialization of nodes.
mod attrs;
mod content;
pub mod helper;
mod schema;

#[cfg(feature = "cmark")]
mod from_markdown;
#[cfg(feature = "cmark")]
mod to_markdown;

use crate::model::{
    AttrNode, Block, Fragment, Leaf, Mark, MarkSet, MarkType, Node, Text, TextNode,
};
pub use attrs::{
    BulletListAttrs, CodeBlockAttrs, HeadingAttrs, ImageAttrs, LinkAttrs, OrderedListAttrs,
};
pub use content::MarkdownContentMatch;
pub use schema::{MarkdownNodeType, MD};

#[cfg(feature = "cmark")]
pub use from_markdown::{from_markdown, FromMarkdownError};
#[cfg(feature = "cmark")]
pub use to_markdown::{to_markdown, ToMarkdownError};

use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// The node type for the markdown schema
#[derive(Debug, Derivative, Deserialize, Serialize, PartialEq, Eq)]
#[derivative(Clone(bound = ""))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MarkdownNode {
    /// The document root
    Doc(Block<MD>),
    /// A heading, e.g. `<h1>`
    Heading(AttrNode<MD, HeadingAttrs>),
    /// A code block
    CodeBlock(AttrNode<MD, CodeBlockAttrs>),
    /// A text node
    Text(TextNode<MD>),
    /// A blockquote
    Blockquote(Block<MD>),
    /// A paragraph
    Paragraph(Block<MD>),
    /// A bullet list
    BulletList(AttrNode<MD, BulletListAttrs>),
    /// An ordered list
    OrderedList(AttrNode<MD, OrderedListAttrs>),
    /// A list item
    ListItem(Block<MD>),
    /// A horizontal line `<hr>`
    HorizontalRule,
    /// A hard break `<br>`
    HardBreak,
    /// An image `<img>`
    Image(Leaf<ImageAttrs>),
}

impl From<TextNode<MD>> for MarkdownNode {
    fn from(text_node: TextNode<MD>) -> Self {
        Self::Text(text_node)
    }
}

impl Node<MD> for MarkdownNode {
    fn text_node(&self) -> Option<&TextNode<MD>> {
        if let Self::Text(node) = self {
            Some(node)
        } else {
            None
        }
    }

    fn new_text_node(node: TextNode<MD>) -> Self {
        Self::Text(node)
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

    fn r#type(&self) -> MarkdownNodeType {
        match self {
            Self::Doc { .. } => MarkdownNodeType::Doc,
            Self::Paragraph { .. } => MarkdownNodeType::Paragraph,
            Self::Blockquote { .. } => MarkdownNodeType::Blockquote,
            Self::HorizontalRule => MarkdownNodeType::HorizontalRule,
            Self::Heading { .. } => MarkdownNodeType::Heading,
            Self::CodeBlock { .. } => MarkdownNodeType::CodeBlock,
            Self::OrderedList { .. } => MarkdownNodeType::OrderedList,
            Self::BulletList { .. } => MarkdownNodeType::BulletList,
            Self::ListItem { .. } => MarkdownNodeType::ListItem,
            Self::Text { .. } => MarkdownNodeType::Text,
            Self::Image { .. } => MarkdownNodeType::Image,
            Self::HardBreak => MarkdownNodeType::HardBreak,
        }
    }

    fn text<A: Into<String>>(text: A) -> Self {
        Self::Text(TextNode {
            text: Text::from(text.into()),
            marks: MarkSet::<MD>::default(),
        })
    }

    fn content(&self) -> Option<&Fragment<MD>> {
        match self {
            Self::Doc(doc) => Some(&doc.content),
            Self::Heading(AttrNode { content, .. }) => Some(content),
            Self::CodeBlock(AttrNode { content, .. }) => Some(content),
            Self::Text { .. } => None,
            Self::Blockquote(Block { content }) => Some(content),
            Self::Paragraph(Block { content }) => Some(content),
            Self::BulletList(AttrNode { content, .. }) => Some(content),
            Self::OrderedList(AttrNode { content, .. }) => Some(content),
            Self::ListItem(Block { content }) => Some(content),
            Self::HorizontalRule => None,
            Self::HardBreak => None,
            Self::Image { .. } => None,
        }
    }

    fn marks(&self) -> Option<&MarkSet<MD>> {
        None
    }

    fn mark(&self, set: MarkSet<MD>) -> Self {
        // TODO: marks on other nodes
        if let Some(text_node) = self.text_node() {
            Self::Text(TextNode {
                marks: set,
                text: text_node.text.clone(),
            })
        } else {
            self.clone()
        }
    }

    fn copy<F>(&self, map: F) -> Self
    where
        F: FnOnce(&Fragment<MD>) -> Fragment<MD>,
    {
        match self {
            Self::Doc(block) => Self::Doc(block.copy(map)),
            Self::Heading(node) => Self::Heading(node.copy(map)),
            Self::CodeBlock(node) => Self::CodeBlock(node.copy(map)),
            Self::Text(node) => Self::Text(node.clone()),
            Self::Blockquote(block) => Self::Blockquote(block.copy(map)),
            Self::Paragraph(block) => Self::Paragraph(block.copy(map)),
            Self::BulletList(node) => Self::BulletList(node.copy(map)),
            Self::OrderedList(node) => Self::OrderedList(node.copy(map)),
            Self::ListItem(block) => Self::ListItem(block.copy(map)),
            Self::HorizontalRule => Self::HorizontalRule,
            Self::HardBreak => Self::HardBreak,
            Self::Image(img) => Self::Image(img.clone()),
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

impl Mark<MD> for MarkdownMark {
    fn r#type(&self) -> MarkdownMarkType {
        match self {
            Self::Strong => MarkdownMarkType::Strong,
            Self::Em => MarkdownMarkType::Em,
            Self::Code => MarkdownMarkType::Code,
            Self::Link { .. } => MarkdownMarkType::Link,
        }
    }
}

/// The type of a markdown mark.
#[derive(Debug, Hash, Eq, Copy, Clone, PartialEq, PartialOrd, Ord)]
pub enum MarkdownMarkType {
    /// bold
    Strong,
    /// italics
    Em,
    /// monospace
    Code,
    /// hyper-linked
    Link,
}

impl MarkType for MarkdownMarkType {}
