use crate::markdown::{MarkdownContentMatch, MarkdownMark, MarkdownNode};
use crate::model::{ContentMatch, Fragment, MarkSet, Node, NodeType, Schema};

/// The markdown schema type
pub struct MD;

impl Schema for MD {
    type Node = MarkdownNode;
    type Mark = MarkdownMark;
    type NodeType = MarkdownNodeType;
    type ContentMatch = MarkdownContentMatch;
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
    pub(crate) fn is_block(self) -> bool {
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

    pub(crate) fn is_inline(self) -> bool {
        matches!(self, Self::Text | Self::Image | Self::HardBreak)
    }
}

impl NodeType<MD> for MarkdownNodeType {
    fn allow_marks(self, marks: &MarkSet<MD>) -> bool {
        true //todo!()
    }

    fn content_match(self) -> MarkdownContentMatch {
        match self {
            Self::Doc => MarkdownContentMatch::BlockPlus,
            Self::Heading => MarkdownContentMatch::OrTextImageStar,
            Self::CodeBlock => MarkdownContentMatch::TextStar,
            Self::Text => MarkdownContentMatch::Empty,
            Self::Blockquote => MarkdownContentMatch::BlockPlus,
            Self::Paragraph => MarkdownContentMatch::InlineStar,
            Self::BulletList => MarkdownContentMatch::ListItemPlus,
            Self::OrderedList => MarkdownContentMatch::ListItemPlus,
            Self::ListItem => MarkdownContentMatch::ParagraphBlockStar,
            Self::HorizontalRule => MarkdownContentMatch::Empty,
            Self::HardBreak => MarkdownContentMatch::Empty,
            Self::Image => MarkdownContentMatch::Empty,
        }
    }

    fn compatible_content(self, other: Self) -> bool {
        self == other || self.content_match().compatible(other.content_match())
    }

    /// Returns true if the given fragment is valid content for this node type with the given
    /// attributes.
    fn valid_content(self, fragment: &Fragment<MD>) -> bool {
        let result = self.content_match().match_fragment(fragment);

        if let Some(m) = result {
            if m.valid_end() {
                for child in fragment.children() {
                    if child.marks().filter(|m| !self.allow_marks(m)).is_some() {
                        return false;
                    }
                }

                return true;
            }
        }

        false
    }
}
