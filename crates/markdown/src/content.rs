use crate::{MarkdownNodeType, MD};
use prosemirror_model::{ContentMatch, NodeType};

/// The content match type for markdown
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MarkdownContentMatch {
    /// `inline*`
    InlineStar,
    /// `block+`
    BlockPlus,
    /// `block*`
    BlockStar,
    /// `(text | image)*`
    OrTextImageStar,
    /// `text*`
    TextStar,
    /// `list_item+`
    ListItemPlus,
    /// `list_item*`
    ListItemStar,
    /// `paragraph block*`
    ParagraphBlockStar,
    /// empty
    Empty,
}

impl ContentMatch<MD> for MarkdownContentMatch {
    fn match_type(self, r#type: MarkdownNodeType) -> Option<Self> {
        match self {
            Self::InlineStar => r#type.is_inline().then(|| Self::InlineStar),
            Self::BlockPlus | Self::BlockStar => r#type.is_block().then(|| Self::BlockStar),
            Self::OrTextImageStar => {
                matches!(r#type, MarkdownNodeType::Text | MarkdownNodeType::Image)
                    .then(|| Self::OrTextImageStar)
            }
            Self::TextStar => matches!(r#type, MarkdownNodeType::Text).then(|| Self::TextStar),
            Self::ListItemPlus | Self::ListItemStar => {
                matches!(r#type, MarkdownNodeType::ListItem).then(|| Self::ListItemStar)
            }
            Self::ParagraphBlockStar => {
                matches!(r#type, MarkdownNodeType::Paragraph).then(|| Self::BlockStar)
            }
            Self::Empty => None,
        }
    }

    fn valid_end(self) -> bool {
        matches!(
            self,
            Self::InlineStar
                | Self::BlockStar
                | Self::OrTextImageStar
                | Self::TextStar
                | Self::ListItemStar
                | Self::Empty
        )
    }
}

impl MarkdownContentMatch {
    pub(crate) fn compatible(self, other: Self) -> bool {
        match self {
            Self::InlineStar => matches!(
                other,
                Self::InlineStar | Self::OrTextImageStar | Self::TextStar
            ),
            Self::BlockPlus | Self::BlockStar => matches!(
                other,
                Self::BlockPlus | Self::ParagraphBlockStar | Self::BlockStar
            ),
            Self::OrTextImageStar => matches!(
                other,
                Self::InlineStar | Self::OrTextImageStar | Self::TextStar
            ),
            Self::TextStar => matches!(
                other,
                Self::InlineStar | Self::OrTextImageStar | Self::TextStar
            ),
            Self::ListItemPlus | Self::ListItemStar => {
                matches!(other, Self::ListItemPlus | Self::ListItemStar)
            }
            Self::ParagraphBlockStar => matches!(other, Self::BlockPlus | Self::ParagraphBlockStar),
            Self::Empty => false,
        }
    }
}
