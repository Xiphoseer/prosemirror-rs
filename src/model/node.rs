use super::{util, Fragment, MarkSet, ResolveErr, ResolvedPos, Schema};
use serde::{Deserialize, Serialize, Serializer};
use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::RangeBounds;

/// This class represents a node in the tree that makes up a ProseMirror document. So a document is
/// an instance of Node, with children that are also instances of Node.
pub trait Node<S: Schema<Node = Self> + 'static>:
    Serialize + for<'de> Deserialize<'de> + Clone + Debug + PartialEq + Eq
{
    /// Create a copy of this node with only the content between the given positions.
    fn cut<R: RangeBounds<usize>>(&self, range: R) -> Cow<Self> {
        let from = util::from(&range);

        if let Some((text, marks)) = self.text_node() {
            let len = text.len_utf16;
            let to = util::to(&range, len);

            if from == 0 && to == len {
                return Cow::Borrowed(self);
            }
            let (_, rest) = util::split_at_utf16(&text.content, from);
            let (rest, _) = util::split_at_utf16(rest, to - from);

            Cow::Owned(Self::new_text_node(
                Text::from(rest.to_owned()),
                marks.clone(),
            ))
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
        if let Some((text, _)) = self.text_node() {
            text.content.clone()
        } else {
            let mut buf = String::new();
            if let Some(c) = self.content() {
                c.text_between(&mut buf, true, 0, c.size(), Some(""), None);
            }
            buf
        }
    }

    /// Represents `.content.size` in JS
    fn content_size(&self) -> usize {
        self.content().map(Fragment::size).unwrap_or(0)
    }

    /// Get the text and marks if this is a text node
    fn text_node(&self) -> Option<(&Text, &MarkSet<S>)>;

    /// Create a new text node
    fn new_text_node(text: Text, marks: MarkSet<S>) -> Self;

    /// Creates a new text node
    fn text<A: Into<String>>(text: A) -> Self;

    /// A container holding the node's children.
    fn content(&self) -> Option<&Fragment<S>>;

    /// Get the child node at the given index. Raises an error when the index is out of range.
    fn child(&self, index: usize) -> Option<&Self> {
        self.content().and_then(|c| c.child(index))
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
                if let Some((text, _)) = self.text_node() {
                    text.len_utf16
                } else {
                    1
                }
            }
        }
    }
}

/// A string that stores its length in utf-16
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
