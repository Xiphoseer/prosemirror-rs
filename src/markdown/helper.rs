//! # Helpers
//!
//! This module contains some functions to create nodes programmatically.
//!
//! See also: <https://github.com/prosemirror/prosemirror-test-builder>
use super::{BulletListAttrs, CodeBlockAttrs, HeadingAttrs, MarkdownMark, MarkdownNode, MD};
use crate::model::{self, AttrNode, Block, Text, TextNode};

type Fragment = model::Fragment<MD>;

impl From<&str> for Fragment {
    fn from(s: &str) -> Fragment {
        Fragment::from(vec![MarkdownNode::from(s)])
    }
}

impl From<MarkdownNode> for Fragment {
    fn from(node: MarkdownNode) -> Fragment {
        Fragment::from(vec![node])
    }
}

/// Create a document node.
pub fn doc<A: Into<Fragment>>(content: A) -> MarkdownNode {
    MarkdownNode::Doc(Block {
        content: content.into(),
    })
}

/// Create a heading node.
pub fn h<A: Into<Fragment>>(level: u8, content: A) -> MarkdownNode {
    MarkdownNode::Heading(AttrNode {
        attrs: HeadingAttrs { level },
        content: content.into(),
    })
}

/// Create a heading (level 1) node.
pub fn h1<A: Into<Fragment>>(content: A) -> MarkdownNode {
    h(1, content)
}

/// Create a heading (level 2) node.
pub fn h2<A: Into<Fragment>>(content: A) -> MarkdownNode {
    h(2, content)
}

/// Create an emphasized text node.
pub fn em(content: &str) -> MarkdownNode {
    MarkdownNode::Text(TextNode {
        text: Text::from(content.to_string()),
        marks: [MarkdownMark::Em].iter().cloned().collect(),
    })
}

/// Create an emphasized text node.
pub fn strong(content: &str) -> MarkdownNode {
    MarkdownNode::Text(TextNode {
        text: Text::from(content.to_string()),
        marks: [MarkdownMark::Strong].iter().cloned().collect(),
    })
}

/// Create a paragraph node.
pub fn p<A: Into<Fragment>>(content: A) -> MarkdownNode {
    MarkdownNode::Paragraph(Block {
        content: content.into(),
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
pub fn blockquote<A: Into<Fragment>>(content: A) -> MarkdownNode {
    MarkdownNode::Blockquote(Block {
        content: content.into(),
    })
}

/// Create a node.
pub fn node<A: Into<MarkdownNode>>(src: A) -> MarkdownNode {
    src.into()
}
