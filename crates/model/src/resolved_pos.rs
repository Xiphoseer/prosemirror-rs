use super::{fragment::IndexError, Fragment, Node, Schema};
use derivative::Derivative;
use derive_new::new;
use displaydoc::Display;
use std::borrow::Cow;
use std::fmt;
use thiserror::Error;

/// Errors at `resolve`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Display, Error)]
pub enum ResolveErr {
    /// Position {pos} out of range
    RangeError {
        /// The position that was out of range
        pos: usize,
    },
    /// Index error
    Index(#[from] IndexError),
}

#[derive(Derivative, new)]
#[derivative(PartialEq(bound = ""), Eq(bound = ""))]
/// A node in the resolution path
pub struct ResolvedNode<'a, S: Schema> {
    /// Reference to the node
    pub node: &'a S::Node,
    /// Index of the in the parent fragment
    pub index: usize,
    /// Offset immediately before the node
    pub before: usize,
}

impl<'a, S: Schema> Clone for ResolvedNode<'a, S> {
    fn clone(&self) -> Self {
        Self {
            node: self.node,
            index: self.index,
            before: self.before,
        }
    }
}

impl<'a, S: Schema> fmt::Debug for ResolvedNode<'a, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedNode")
            .field("node.type", &self.node.r#type())
            .field("index", &self.index)
            .field("before", &self.before)
            .finish()
    }
}

/// You can resolve a position to get more information about it. Objects of this class represent
/// such a resolved position, providing various pieces of context information, and some helper
/// methods.
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub struct ResolvedPos<'a, S: Schema> {
    pub(crate) pos: usize,
    path: Vec<ResolvedNode<'a, S>>,
    pub(crate) parent_offset: usize,
    pub(crate) depth: usize,
}

impl<'a, S: Schema> ResolvedPos<'a, S> {
    pub fn depth(&self) -> usize {
        self.depth
    }

    pub(crate) fn new(pos: usize, path: Vec<ResolvedNode<'a, S>>, parent_offset: usize) -> Self {
        Self {
            depth: path.len() - 1,
            pos,
            path,
            parent_offset,
        }
    }

    /// The parent node that the position points into. Note that even if
    /// a position points into a text node, that node is not considered
    /// the parent—text nodes are ‘flat’ in this model, and have no content.
    pub fn parent(&self) -> &S::Node {
        self.node(self.depth)
    }

    /// The root node in which the position was resolved.
    pub fn doc(&self) -> &S::Node {
        self.node(0)
    }

    /// The ancestor node at the given level. `p.node(p.depth)` is the same as `p.parent()`.
    pub fn node(&self, depth: usize) -> &'a S::Node {
        self.path[depth].node
    }

    /// The index into the ancestor at the given level. If this points at the 3rd node in the
    /// 2nd paragraph on the top level, for example, `p.index(0)` is 1 and `p.index(1)` is 2.
    pub fn index(&self, depth: usize) -> usize {
        self.path[depth].index
    }

    /// The index pointing after this position into the ancestor at the given level.
    pub fn index_after(&self, depth: usize) -> usize {
        let index = self.index(depth);
        if depth == self.depth && self.text_offset() == 0 {
            index
        } else {
            index + 1
        }
    }

    /// The (absolute) position at the start of the node at the given level.
    pub fn start(&self, depth: usize) -> usize {
        if depth == 0 {
            0
        } else {
            self.path[depth - 1].before + 1
        }
    }

    /// The (absolute) position at the end of the node at the given level.
    pub fn end(&self, depth: usize) -> usize {
        self.start(depth) + self.node(depth).content().map(Fragment::size).unwrap_or(0)
    }

    /// The (absolute) position directly before the wrapping node at the given level, or, when
    /// depth is `self.depth + 1`, the original position.
    pub fn before(&self, depth: usize) -> Option<usize> {
        if depth == 0 {
            None
        } else if depth == self.depth + 1 {
            Some(self.pos)
        } else {
            Some(self.path[depth - 1].before)
        }
    }

    /// The (absolute) position directly after the wrapping node at the given level, or the
    /// original position when depth is `self.depth + 1`.
    pub fn after(&self, depth: usize) -> Option<usize> {
        if depth == 0 {
            None
        } else if depth == self.depth + 1 {
            Some(self.pos)
        } else {
            Some(self.path[depth - 1].before + self.path[depth].node.node_size())
        }
    }

    /// When this position points into a text node, this returns the
    /// distance between the position and the start of the text node.
    /// Will be zero for positions that point between nodes.
    pub fn text_offset(&self) -> usize {
        self.pos - self.path.last().unwrap().before
    }

    /// Get the node directly before the position, if any. If the position points into a text node,
    /// only the part of that node before the position is returned.
    pub fn node_before(&self) -> Option<Cow<S::Node>> {
        let index = self.index(self.depth);
        let d_off = self.pos - self.path.last().unwrap().before;
        if d_off > 0 {
            let parent = self.parent();
            let child = parent.child(index).unwrap();
            let cut = child.cut(0..d_off);
            Some(cut)
        } else if index == 0 {
            None
        } else {
            Some(Cow::Borrowed(self.parent().child(index - 1).unwrap()))
        }
    }

    /// Get the node directly after the position, if any. If the position points into a text node,
    /// only the part of that node after the position is returned.
    pub fn node_after(&self) -> Option<Cow<S::Node>> {
        let parent = self.parent();
        let index = self.index(self.depth);
        if index == parent.child_count() {
            return None;
        }
        let d_off = self.pos - self.path.last().unwrap().before;
        let child = parent.child(index).unwrap();
        if d_off > 0 {
            Some(child.cut(d_off..))
        } else {
            Some(Cow::Borrowed(child))
        }
    }

    /// The depth up to which this position and the given (non-resolved)
    /// position share the same parent nodes.
    pub fn shared_depth(&self, pos: usize) -> usize {
        for depth in (1..=self.depth).rev() {
            if self.start(depth) <= pos && self.end(depth) >= pos {
                return depth;
            }
        }
        0
    }

    pub(crate) fn resolve(doc: &'a S::Node, pos: usize) -> Result<Self, ResolveErr> {
        if pos > doc.content().unwrap().size() {
            return Err(ResolveErr::RangeError { pos });
        }
        let mut path = vec![];
        let mut start = 0;
        let mut parent_offset = pos;
        let mut node = doc;

        loop {
            let Index { index, offset } = node
                .content()
                .unwrap_or(&Fragment::default())
                .find_index(parent_offset, false)?;
            let rem = parent_offset - offset;
            path.push(ResolvedNode {
                node,
                index,
                before: start + offset,
            });
            if rem == 0 {
                break;
            }
            node = node.child(index).unwrap();
            if node.is_text() {
                break;
            }
            parent_offset = rem - 1;
            start += offset + 1;
        }
        Ok(ResolvedPos::new(pos, path, parent_offset))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index {
    pub index: usize,
    pub offset: usize,
}

impl Index {
    #[allow(unused)]
    pub fn new(index: usize, offset: usize) -> Index {
        Index { index, offset }
    }
}
