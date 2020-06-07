use super::{Fragment, Node};
use std::borrow::Cow;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ResolveErr {
    /// Position {pos} out of range
    RangeError { pos: usize },
    /// Broken Invariant
    BrokenInvariant,
}

impl From<()> for ResolveErr {
    fn from(_e: ()) -> Self {
        Self::BrokenInvariant
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPos<'a> {
    pub(crate) pos: usize,
    pub(crate) path: Vec<(&'a Node, usize, usize)>,
    pub(crate) parent_offset: usize,
    pub(crate) depth: usize,
}

impl<'a> ResolvedPos<'a> {
    pub fn new(pos: usize, path: Vec<(&'a Node, usize, usize)>, parent_offset: usize) -> Self {
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
    pub fn parent(&self) -> &Node {
        self.node(self.depth)
    }

    /// The root node in which the position was resolved.
    pub fn doc(&self) -> &Node {
        self.node(0)
    }

    pub fn node(&self, depth: usize) -> &Node {
        self.path[depth].0
    }

    pub fn index(&self, depth: usize) -> usize {
        self.path[depth].1
    }

    pub fn start(&self, depth: usize) -> usize {
        if depth == 0 {
            0
        } else {
            self.path[depth - 1].2 + 1
        }
    }

    pub fn end(&self, depth: usize) -> usize {
        self.start(depth) + self.node(depth).content().map(Fragment::size).unwrap_or(0)
    }

    pub fn before(&self, depth: usize) -> Option<usize> {
        if depth == 0 {
            None
        } else if depth == self.depth + 1 {
            Some(self.pos)
        } else {
            Some(self.path[depth - 1].2)
        }
    }

    pub fn after(&self, depth: usize) -> Option<usize> {
        if depth == 0 {
            None
        } else if depth == self.depth + 1 {
            Some(self.pos)
        } else {
            Some(self.path[depth - 1].2 + self.path[depth].0.node_size())
        }
    }

    pub fn node_before(&self) -> Option<Cow<Node>> {
        let index = self.index(self.depth);
        let d_off = self.pos - self.path.last().unwrap().2;
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

    pub fn node_after(&self) -> Option<Cow<Node>> {
        let parent = self.parent();
        let index = self.index(self.depth);
        if index == parent.child_count() {
            return None;
        }
        let d_off = self.pos - self.path.last().unwrap().2;
        let child = parent.child(index).unwrap();
        if d_off > 0 {
            Some(child.cut(d_off..))
        } else {
            Some(Cow::Borrowed(child))
        }
    }

    pub fn resolve(doc: &'a Node, pos: usize) -> Result<Self, ResolveErr> {
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
            path.push((node, index, start + offset));
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
pub(crate) struct Index {
    pub index: usize,
    pub offset: usize,
}

impl Index {
    #[allow(unused)]
    pub fn new(index: usize, offset: usize) -> Index {
        Index { index, offset }
    }
}
