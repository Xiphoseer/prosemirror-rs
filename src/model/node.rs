use super::{
    replace, util, Fragment, MarkSet, ReplaceError, ResolveErr, ResolvedPos, Schema, Slice,
    TextNode,
};
use displaydoc::Display;
use serde::{Deserialize, Serialize, Serializer};
use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::RangeBounds;
use thiserror::Error;

#[derive(Debug, Clone, Error, Display, Eq, PartialEq)]
/// Error type raised by `Node::slice` when given an invalid replacement.
pub enum SliceError {
    /// The given span was invalid
    Resolve(#[from] ResolveErr),
    /// Unknown
    Unknown,
}

/// This is the type that encodes a kind of node
pub trait NodeType<S: Schema>: Copy + Clone + Debug + PartialEq + Eq {
    /// ???
    fn compatible_content(self, other: Self) -> bool;
    /// ???
    fn valid_content(self, fragment: &Fragment<S>) -> bool;
}

/// This class represents a node in the tree that makes up a ProseMirror document. So a document is
/// an instance of Node, with children that are also instances of Node.
pub trait Node<S: Schema<Node = Self> + 'static>:
    Serialize + for<'de> Deserialize<'de> + Clone + Debug + PartialEq + Eq + Sized + From<TextNode<S>>
{
    /// Create a copy of this node with only the content between the given positions.
    fn cut<R: RangeBounds<usize>>(&self, range: R) -> Cow<Self> {
        let from = util::from(&range);

        if let Some(TextNode { text, marks }) = self.text_node() {
            let len = text.len_utf16;
            let to = util::to(&range, len);

            if from == 0 && to == len {
                return Cow::Borrowed(self);
            }
            let (_, rest) = util::split_at_utf16(&text.content, from);
            let (rest, _) = util::split_at_utf16(rest, to - from);

            Cow::Owned(Self::new_text_node(TextNode {
                text: Text::from(rest.to_owned()),
                marks: marks.clone(),
            }))
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

    /// Cut out the part of the document between the given positions, and return it as a `Slice` object.
    fn slice<R: RangeBounds<usize> + Debug>(
        &self,
        range: R,
        include_parents: bool,
    ) -> Result<Slice<S>, SliceError> {
        let from = util::from(&range);
        let to = util::to(&range, self.node_size());

        if from == to {
            return Ok(Slice::default());
        }

        let rp_from = self.resolve(from)?;
        let rp_to = self.resolve(to)?;

        let depth = if include_parents {
            0
        } else {
            rp_from.shared_depth(to)
        };

        let (start, node) = (rp_from.start(depth), rp_from.node(depth));
        let content = if let Some(c) = node.content() {
            c.cut(rp_from.pos - start..rp_to.pos - start)
        } else {
            Fragment::new()
        };
        Ok(Slice::new(
            content,
            rp_from.depth - depth,
            rp_to.depth - depth,
        ))
    }

    /// Replace the part of the document between the given positions with the given slice. The
    /// slice must 'fit', meaning its open sides must be able to connect to the surrounding content,
    /// and its content nodes must be valid children for the node they are placed into. If any of
    /// this is violated, an error of type
    /// [`ReplaceError`](#model.ReplaceError) is thrown.
    fn replace<R: RangeBounds<usize> + Debug>(
        &self,
        range: R,
        slice: &Slice<S>,
    ) -> Result<Self, ReplaceError<S>> {
        let from = util::from(&range);
        let to = util::to(&range, self.node_size());
        // FIXME: this max value is my guess, that needs to be tested out

        assert!(to >= from, "replace: {} >= {}", to, from);

        let rp_from = self.resolve(from)?;
        let rp_to = self.resolve(to)?;

        let node = replace(&rp_from, &rp_to, slice)?;
        Ok(node)
    }

    /// Resolve the given position in the document, returning a struct with information about its
    /// context.
    fn resolve(&self, pos: usize) -> Result<ResolvedPos<S>, ResolveErr> {
        ResolvedPos::resolve(self, pos)
    }

    /// Create a new node with the same markup as this node, containing the given content (or
    /// empty, if no content is given).
    fn copy<F>(&self, map: F) -> Self
    where
        F: FnOnce(&Fragment<S>) -> Fragment<S>;

    /// Concatenates all the text nodes found in this fragment and its children.
    fn text_content(&self) -> String {
        if let Some(node) = self.text_node() {
            node.text.content.clone()
        } else {
            let mut buf = String::new();
            if let Some(c) = self.content() {
                c.text_between(&mut buf, true, 0, c.size(), Some(""), None);
            }
            buf
        }
    }

    /// Returns this node's first child wrapped in `Some`, or `Node` if there are no children.
    fn first_child(&self) -> Option<&S::Node> {
        self.content().and_then(Fragment::first_child)
    }

    /// Represents `.content.size` in JS
    fn content_size(&self) -> usize {
        self.content().map(Fragment::size).unwrap_or(0)
    }

    /// Get the text and marks if this is a text node
    fn text_node(&self) -> Option<&TextNode<S>>;

    /// Create a new text node
    fn new_text_node(node: TextNode<S>) -> Self;

    /// Creates a new text node
    fn text<A: Into<String>>(text: A) -> Self;

    /// A container holding the node's children.
    fn content(&self) -> Option<&Fragment<S>>;

    /// Get the marks on this node
    fn marks(&self) -> Option<&MarkSet<S>>;

    /// Get the type of the node
    fn r#type(&self) -> S::NodeType;

    /// Get the child node at the given index. Raises an error when the index is out of range.
    fn child(&self, index: usize) -> Option<&Self> {
        self.content().map(|c| c.child(index))
    }

    /// Get the child node at the given index, if it exists.
    fn maybe_child(&self, index: usize) -> Option<&Self> {
        self.content().and_then(|c| c.maybe_child(index))
    }

    /// The number of children that the node has.
    fn child_count(&self) -> usize {
        self.content().map_or(0, Fragment::child_count)
    }

    /// True when this is a leaf node.
    fn is_leaf(&self) -> bool {
        self.content().is_none()
    }

    /// True when this is a block (non-inline node)
    fn is_block(&self) -> bool;

    /// True when this is a text node.
    fn is_text(&self) -> bool {
        self.text_node().is_some()
    }

    /// The size of this node, as defined by the integer-based indexing scheme. For text nodes,
    /// this is the amount of characters. For other leaf nodes, it is one. For non-leaf nodes, it
    /// is the size of the content plus two (the start and end token).
    fn node_size(&self) -> usize {
        match self.content() {
            Some(c) => c.size() + 2,
            None => {
                if let Some(node) = self.text_node() {
                    node.text.len_utf16
                } else {
                    1
                }
            }
        }
    }
}

/// A string that stores its length in utf-16
#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(from = "String")]
pub struct Text {
    len_utf16: usize,
    content: String,
}

impl Text {
    /// Return the contained string
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// The length of this string if it were encoded in utf-16
    pub fn len_utf16(&self) -> usize {
        self.len_utf16
    }

    /// Join two texts together
    pub fn join(&self, other: &Self) -> Self {
        let left = &self.content;
        let right = &other.content;
        let mut content = String::with_capacity(left.len() + right.len());
        content.push_str(left);
        content.push_str(right);
        let len_utf16 = self.len_utf16 + other.len_utf16;
        Text { content, len_utf16 }
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
