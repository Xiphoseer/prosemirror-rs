//! # The markdown schema
//!
//! This module is derived from the `prosemirror-markdown` schema and the
//! the general JSON serialization of nodes.
mod attrs;
pub mod helper;

use crate::model::{
    AttrNode, Block, Fragment, Leaf, Mark, MarkSet, Node, NodeType, Schema, Text, TextNode,
};
pub use attrs::{
    BulletListAttrs, CodeBlockAttrs, HeadingAttrs, ImageAttrs, LinkAttrs, OrderedListAttrs,
};
use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// The markdown schema type
pub struct MD;

impl Schema for MD {
    type Node = MarkdownNode;
    type Mark = MarkdownMark;
    type NodeType = MarkdownNodeType;
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ContentMatch {
    /// `inline*`
    InlineStar,
    /// `block+`
    BlockPlus,
    /// `(text | image)*`
    OrTextImageStar,
    /// `text*`
    TextStar,
    /// `list_item+`
    ListItemPlus,
    /// `paragraph block*`
    ParagraphBlockStar,
    /// empty
    Empty,
}

impl ContentMatch {
    fn compatible(self, other: Self) -> bool {
        match self {
            Self::InlineStar => matches!(
                other,
                Self::InlineStar | Self::OrTextImageStar | Self::TextStar
            ),
            Self::BlockPlus => matches!(other, Self::BlockPlus | Self::ParagraphBlockStar),
            Self::OrTextImageStar => matches!(
                other,
                Self::InlineStar | Self::OrTextImageStar | Self::TextStar
            ),
            Self::TextStar => matches!(
                other,
                Self::InlineStar | Self::OrTextImageStar | Self::TextStar
            ),
            Self::ListItemPlus => other == Self::ListItemPlus,
            Self::ParagraphBlockStar => matches!(other, Self::BlockPlus | Self::ParagraphBlockStar),
            Self::Empty => false,
        }
    }

    fn match_fragment(self, fragment: &Fragment<MD>) -> MatchResult {
        match self {
            Self::InlineStar => {
                if fragment.children().iter().all(|n| n.r#type().is_inline()) {
                    MatchResult::ValidEnd
                } else {
                    MatchResult::Invalid
                }
            }
            Self::BlockPlus => {
                if fragment
                    .children()
                    .iter()
                    .map(Node::r#type)
                    .all(MarkdownNodeType::is_block)
                {
                    if fragment.child_count() > 0 {
                        MatchResult::ValidEnd
                    } else {
                        MatchResult::Valid
                    }
                } else {
                    MatchResult::Invalid
                }
            }
            Self::OrTextImageStar => {
                if fragment
                    .children()
                    .iter()
                    .map(Node::r#type)
                    .all(move |n| matches!(n, MarkdownNodeType::Text | MarkdownNodeType::Image))
                {
                    MatchResult::ValidEnd
                } else {
                    MatchResult::Invalid
                }
            }
            Self::TextStar => {
                if fragment
                    .children()
                    .iter()
                    .map(Node::r#type)
                    .all(move |n| n == MarkdownNodeType::Text)
                {
                    MatchResult::ValidEnd
                } else {
                    MatchResult::Invalid
                }
            }
            Self::ListItemPlus => {
                if fragment
                    .children()
                    .iter()
                    .map(Node::r#type)
                    .all(move |n| n == MarkdownNodeType::ListItem)
                {
                    if fragment.child_count() > 0 {
                        MatchResult::ValidEnd
                    } else {
                        MatchResult::Valid
                    }
                } else {
                    MatchResult::Invalid
                }
            }
            Self::ParagraphBlockStar => {
                if let Some((first, rest)) = fragment.children().split_first() {
                    if first.r#type() != MarkdownNodeType::Paragraph {
                        MatchResult::Invalid
                    } else {
                        if rest.iter().all(|n| n.r#type().is_block()) {
                            MatchResult::ValidEnd
                        } else {
                            MatchResult::Invalid
                        }
                    }
                } else {
                    MatchResult::Valid
                }
            }
            Self::Empty => {
                if fragment.children().is_empty() {
                    MatchResult::ValidEnd
                } else {
                    MatchResult::Invalid
                }
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum MatchResult {
    Invalid,
    Valid,
    ValidEnd,
}

/// The node-spec type for the markdown schema
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MarkdownNodeType {
    /// The document root
    Doc,
    /// A heading, e.g. `<h1>`
    Heading,
    /// A code block
    CodeBlock,
    /// A text node
    Text,
    /// A blockquote
    Blockquote,
    /// A paragraph
    Paragraph,
    /// A bullet list
    BulletList,
    /// An ordered list
    OrderedList,
    /// A list item
    ListItem,
    /// A horizontal line `<hr>`
    HorizontalRule,
    /// A hard break `<br>`
    HardBreak,
    /// An image `<img>`
    Image,
}

impl MarkdownNodeType {
    fn content_match(self) -> ContentMatch {
        match self {
            Self::Doc => ContentMatch::BlockPlus,
            Self::Heading => ContentMatch::OrTextImageStar,
            Self::CodeBlock => ContentMatch::TextStar,
            Self::Text => ContentMatch::Empty,
            Self::Blockquote => ContentMatch::BlockPlus,
            Self::Paragraph => ContentMatch::InlineStar,
            Self::BulletList => ContentMatch::ListItemPlus,
            Self::OrderedList => ContentMatch::ListItemPlus,
            Self::ListItem => ContentMatch::ParagraphBlockStar,
            Self::HorizontalRule => ContentMatch::Empty,
            Self::HardBreak => ContentMatch::Empty,
            Self::Image => ContentMatch::Empty,
        }
    }

    fn allow_marks(self, marks: &MarkSet<MD>) -> bool {
        true //todo!()
    }

    fn is_block(self) -> bool {
        matches!(
            self,
            Self::Paragraph
                | Self::Blockquote
                | Self::Heading
                | Self::HorizontalRule
                | Self::CodeBlock
                | Self::OrderedList
                | Self::BulletList
        )
    }

    fn is_inline(self) -> bool {
        matches!(self, Self::Text | Self::Image | Self::HardBreak)
    }
}

impl NodeType<MD> for MarkdownNodeType {
    fn compatible_content(self, other: Self) -> bool {
        self == other || self.content_match().compatible(other.content_match())
    }

    /// Returns true if the given fragment is valid content for this node type with the given
    /// attributes.
    fn valid_content(self, fragment: &Fragment<MD>) -> bool {
        let result = self.content_match().match_fragment(fragment);

        if matches!(result, MatchResult::ValidEnd) {
            for child in fragment.children() {
                if child.marks().filter(|m| !self.allow_marks(m)).is_some() {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }
}

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
            marks: MarkSet::<MD>::new(),
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

impl Mark for MarkdownMark {}
