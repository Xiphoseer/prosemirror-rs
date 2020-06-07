use super::{util, Index, Node};
use serde::{Deserialize, Serialize, Serializer};
use std::ops::RangeBounds;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(from = "Vec<Node>")]
pub struct Fragment {
    inner: Vec<Node>,
    size: usize,
}

impl Fragment {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn count(&self) -> usize {
        self.inner.len()
    }

    pub fn cut<R: RangeBounds<usize>>(&self, range: R) -> Fragment {
        let from = util::from(&range);
        let to = util::to(&range, self.size);

        if from == 0 && to == self.size {
            return self.clone();
        }

        let mut result = vec![];
        let mut size = 0;
        if to > from {
            let mut pos = 0;
            let mut i = 0;
            while pos < to {
                let child = &self.inner[i];
                let end = pos + child.node_size();
                if end > from {
                    let new_child = if pos < from || end > to {
                        if let Node::Text { text, .. } = child {
                            child.cut(
                                usize::max(0, from - pos)..usize::min(text.len_utf16(), to - pos),
                            )
                        } else {
                            child.cut(
                                usize::max(0, from - pos - 1)
                                    ..usize::min(child.content_size(), to - pos - 1),
                            )
                        }
                        .into_owned()
                    } else {
                        child.clone()
                    };
                    result.push(new_child);
                    size += child.node_size();
                }
                pos = end;
                i += 1;
            }
        }
        Fragment {
            inner: result,
            size,
        }
    }

    pub fn nodes_between<F: FnMut(&Node, usize) -> bool>(
        &self,
        from: usize,
        to: usize,
        f: &mut F,
        node_start: usize,
    ) {
        let mut pos = 0;
        for child in &self.inner {
            let end = pos + child.node_size();
            if end > from && f(child, node_start + pos) {
                if let Some(content) = child.content() {
                    let start = pos + 1;
                    content.nodes_between(
                        usize::max(0, from - start),
                        usize::min(content.size(), to - start),
                        f,
                        node_start + start,
                    )
                }
            }
            pos = end;
        }
    }

    pub fn text_between(
        &self,
        text: &mut String,
        mut separated: bool,
        from: usize,
        to: usize,
        block_separator: Option<&str>,
        leaf_text: Option<&str>,
    ) {
        self.nodes_between(
            from,
            to,
            &mut move |node, pos| {
                if let Node::Text { text: txt, .. } = node {
                    let (rest, skip) = if from > pos {
                        let skip = from - pos;
                        (util::split_at_utf16(txt.as_str(), skip).1, skip)
                    } else {
                        (txt.as_str(), 0)
                    };

                    let end = to - pos;
                    let slice = util::split_at_utf16(rest, end - skip).0;

                    text.push_str(slice);
                    separated = block_separator.is_none();
                } else if node.is_leaf() {
                    if let Some(leaf_text) = leaf_text {
                        text.push_str(leaf_text);
                    }
                    separated = block_separator.is_none();
                } else if !separated && node.is_block() {
                    text.push_str(block_separator.unwrap_or(""));
                    separated = true
                }
                true
            },
            0,
        )
    }

    pub(crate) fn child(&self, index: usize) -> Option<&Node> {
        self.inner.get(index)
    }

    pub(crate) fn find_index(&self, pos: usize, round: bool) -> Result<Index, ()> {
        let len = self.inner.len();
        match pos {
            0 => Ok(Index {
                index: 0,
                offset: pos,
            }),
            p if p == self.size => Ok(Index {
                index: len,
                offset: pos,
            }),
            p if p > self.size => Err(()),
            p => {
                let mut cur_pos = 0;
                for (i, cur) in self.inner.iter().enumerate() {
                    let end = cur_pos + cur.node_size();
                    if end >= p {
                        if (end == p) || round {
                            return Ok(Index {
                                index: i + 1,
                                offset: end,
                            });
                        } else {
                            return Ok(Index {
                                index: i,
                                offset: cur_pos,
                            });
                        }
                    }
                    cur_pos = end;
                }
                panic!("Invariant failed: self.size must be the sum of all node sizes")
            }
        }
    }
}

impl Default for Fragment {
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            size: 0,
        }
    }
}

impl Serialize for Fragment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl From<Vec<Node>> for Fragment {
    fn from(src: Vec<Node>) -> Fragment {
        let size = src.iter().map(|x| x.node_size()).sum::<usize>();
        Fragment { inner: src, size }
    }
}

impl From<Fragment> for Vec<Node> {
    fn from(src: Fragment) -> Vec<Node> {
        src.inner
    }
}

impl<A, B> From<(A, B)> for Fragment
where
    A: Into<Node>,
    B: Into<Node>,
{
    fn from((a, b): (A, B)) -> Self {
        Self::from(vec![a.into(), b.into()])
    }
}

impl<A: Into<Node>> From<A> for Fragment {
    fn from(a: A) -> Self {
        Self::from(vec![a.into()])
    }
}
