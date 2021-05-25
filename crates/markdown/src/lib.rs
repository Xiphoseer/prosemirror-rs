//! # The markdown schema
//!
//! This module is derived from the `prosemirror-markdown` schema and the
//! the general JSON serialization of nodes.
mod attrs;
mod content;
mod de;
pub mod helper;
mod schema;

#[cfg(feature = "cmark")]
mod from_markdown;
#[cfg(feature = "cmark")]
mod to_markdown;

use prosemirror_model::{
    AttrNode, Block, Fragment, Leaf, Mark, MarkSet, MarkType, Node, Text, TextNode,
    NodeImpl,
};
pub use attrs::{
    BulletListAttrs, CodeBlockAttrs, HeadingAttrs, ImageAttrs, LinkAttrs, OrderedListAttrs,
};
use prosemirror_derive::Node;
pub use content::MarkdownContentMatch;
pub use schema::{MarkdownNodeType, MD};

#[cfg(feature = "cmark")]
pub use from_markdown::{from_markdown, FromMarkdownError};
#[cfg(feature = "cmark")]
pub use to_markdown::{to_markdown, ToMarkdownError};

//use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// The node type for the markdown schema
#[derive(Node, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[prosemirror(schema = MD)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MarkdownNode {
    /// The document root
    Doc {
        content: Fragment<MD>
    },

    /// A paragraph
    #[prosemirror(group = Block)]
    Paragraph(Block<MD>),

    /// A blockquote
    #[prosemirror(group = Block)]
    Blockquote(Block<MD>),

    /// A horizontal line `<hr>`
    #[prosemirror(group = Block)]
    HorizontalRule,

    /// A heading, e.g. `<h1>`
    #[prosemirror(group = Block)]
    Heading(AttrNode<MD, HeadingAttrs>),
    
    /// A code block
    #[prosemirror(group = Block)]
    CodeBlock(AttrNode<MD, CodeBlockAttrs>),

    /// An ordered list
    #[prosemirror(group = Block)]
    OrderedList(AttrNode<MD, OrderedListAttrs>),
    
    /// A bullet list
    #[prosemirror(group = Block)]
    BulletList(AttrNode<MD, BulletListAttrs>),
    
    /// A list item
    #[prosemirror(defining)]
    ListItem(Block<MD>),

    /// A text node
    #[prosemirror(group = Inline)]
    Text(TextNode<MD>),

    /// An image `<img>`
    #[prosemirror(inline)]
    Image(Leaf<ImageAttrs>),

    /// A hard break `<br>`
    #[prosemirror(inline)]
    HardBreak,
}

impl From<TextNode<MD>> for MarkdownNode {
    fn from(text_node: TextNode<MD>) -> Self {
        Self::Text(text_node)
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
