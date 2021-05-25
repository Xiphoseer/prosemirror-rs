//! # Helpers
//!
//! This module contains some functions to create nodes programmatically.
//!
//! See also: <https://github.com/prosemirror/prosemirror-test-builder>
use super::{BulletListAttrs, CodeBlockAttrs, HeadingAttrs, MarkdownMark, MarkdownNode, MD};
use prosemirror_model::{self, AttrNode, Block, Mark, Text, TextNode};

type Fragment = prosemirror_model::Fragment<MD>;

pub trait IntoFragment {
    fn into_fragment(self) -> Fragment;
}

impl IntoFragment for &str {
    fn into_fragment(self) -> Fragment {
        Fragment::from(vec![MarkdownNode::from(self)])
    }
}

impl IntoFragment for MarkdownNode {
    fn into_fragment(self) -> Fragment {
        Fragment::from(vec![self])
    }
}

impl IntoFragment for Vec<MarkdownNode> {
    fn into_fragment(self) -> Fragment {
        Fragment::from(self)
    }
}

impl<A, B> IntoFragment for (A, B)
where
    A: Into<MarkdownNode>,
    B: Into<MarkdownNode>,
{
    fn into_fragment(self) -> Fragment {
        Fragment::from(vec![self.0.into(), self.1.into()])
    }
}

impl<A> IntoFragment for (A,)
where
    A: Into<MarkdownNode>,
{
    fn into_fragment(self) -> Fragment {
        Fragment::from(vec![self.0.into()])
    }
}

/*impl From<&str> for Fragment {
    fn from(s: &str) -> Fragment {
        Fragment::from(vec![MarkdownNode::from(s)])
    }
}*/

impl From<MarkdownNode> for Fragment {
    fn from(node: MarkdownNode) -> Fragment {
        Fragment::from(vec![node])
    }
}

/// Create a document node.
pub fn doc<A: IntoFragment>(content: A) -> MarkdownNode {
    MarkdownNode::Doc(Block {
        content: content.into_fragment(),
    })
}

/// Create a heading node.
pub fn h<A: IntoFragment>(level: u8, content: A) -> MarkdownNode {
    MarkdownNode::Heading(AttrNode {
        attrs: HeadingAttrs { level },
        content: content.into_fragment(),
    })
}

/// Create a heading (level 1) node.
pub fn h1<A: IntoFragment>(content: A) -> MarkdownNode {
    h(1, content)
}

/// Create a heading (level 2) node.
pub fn h2<A: IntoFragment>(content: A) -> MarkdownNode {
    h(2, content)
}

/// Create an emphasized text node.
pub fn em(content: &str) -> MarkdownNode {
    MarkdownNode::Text(TextNode {
        text: Text::from(content.to_string()),
        marks: MarkdownMark::Em.into_set(),
    })
}

/// Create an emphasized text node.
pub fn strong(content: &str) -> MarkdownNode {
    MarkdownNode::Text(TextNode {
        text: Text::from(content.to_string()),
        marks: MarkdownMark::Strong.into_set(),
    })
}

/// Create a paragraph node.
pub fn p<A: IntoFragment>(content: A) -> MarkdownNode {
    MarkdownNode::Paragraph(Block {
        content: content.into_fragment(),
    })
}

/// Create a list item node.
pub fn li<A: Into<Fragment>>(content: A) -> MarkdownNode {
    MarkdownNode::ListItem(Block {
        content: content.into(),
    })
}

/// Create a Bullet list node.
pub fn ul<A: Into<Fragment>>(content: A) -> MarkdownNode {
    MarkdownNode::BulletList(AttrNode {
        attrs: BulletListAttrs { tight: false },
        content: content.into(),
    })
}

/// Create a code block node.
pub fn code_block<A: Into<Fragment>>(params: &str, content: A) -> MarkdownNode {
    MarkdownNode::CodeBlock(AttrNode {
        attrs: CodeBlockAttrs {
            params: params.to_owned(),
        },
        content: content.into(),
    })
}

/// Create a blockquote node.
pub fn blockquote<A: IntoFragment>(content: A) -> MarkdownNode {
    MarkdownNode::Blockquote(Block {
        content: content.into_fragment(),
    })
}

/// Create a node.
pub fn node<A: Into<MarkdownNode>>(src: A) -> MarkdownNode {
    src.into()
}
