use super::{fragment::IndexError, Index};
use crate::{ContentMatchError, Fragment, Node, NodeType, ResolveErr, ResolvedPos, Schema};
use crate::util::EitherOrBoth;
use derivative::Derivative;
use displaydoc::Display;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use thiserror::Error;

/// A slice of a fragment
#[derive(Debug, Derivative, Deserialize, Serialize)]
#[derivative(
    PartialEq(bound = ""),
    Eq(bound = ""),
    Default(bound = "")
)]
#[serde(bound = "", rename_all = "camelCase")]
pub struct Slice<S: Schema> {
    /// The slice's content.
    pub content: Fragment<S>,
    /// The open depth at the start.
    #[serde(default)]
    pub open_start: usize,
    /// The open depth at the end.
    #[serde(default)]
    pub open_end: usize,
}

impl<S: Schema> Slice<S> {
    /// Create a slice. When specifying a non-zero open depth, you must
    /// make sure that there are nodes of at least that depth at the
    /// appropriate side of the fragment — i.e. if the fragment is an empty
    /// paragraph node, `openStart` and `openEnd` can't be greater than 1.
    ///
    /// It is not necessary for the content of open nodes to conform to
    /// the schema's content constraints, though it should be a valid
    /// start/end/middle for such a node, depending on which sides are
    /// open.
    pub fn new(content: Fragment<S>, open_start: usize, open_end: usize) -> Slice<S> {
        Slice {
            content,
            open_start,
            open_end,
        }
    }

    pub fn insert_at(
        &self,
        pos: usize,
        fragment: Fragment<S>,
    ) -> Result<Option<Slice<S>>, InsertError> {
        let content = insert_into(&self.content, pos + self.open_start, fragment, None)?;
        Ok(content.map(|c| Slice::<S>::new(c, self.open_start, self.open_end)))
    }
}

/// Error on insertion
#[derive(Debug, Display, Error)]
pub enum InsertError {
    /// Index error
    Index(#[from] IndexError),
    /// Content match error
    Content(#[from] ContentMatchError),
}

fn insert_into<S: Schema>(
    content: &Fragment<S>,
    dist: usize,
    insert: Fragment<S>,
    parent: Option<&S::Node>,
) -> Result<Option<Fragment<S>>, InsertError> {
    let Index { index, offset } = content.find_index(dist, false)?;
    let child = content.maybe_child(index);
    if offset == dist || matches!(child, Some(c) if c.is_text()) {
        if let Some(p) = parent {
            if !p.can_replace(index, index, Some(&insert), ..)? {
                return Ok(None);
            }
        }

        Ok(Some(
            content
                .cut(..dist)
                .append(insert)
                .append(content.cut(dist..)),
        ))
    } else {
        let child = child.unwrap(); // supposed to be safe, because of offset != diff
        let inner = insert_into(
            child.content().unwrap_or(Fragment::EMPTY_REF),
            dist - offset - 1,
            insert,
            None,
        )?;
        if let Some(i) = inner {
            Ok(Some(
                content.replace_child(index, child.copy(|_| i)).into_owned(),
            ))
        } else {
            Ok(None)
        }
    }
}

/// An error that can occur when replacing a slice
#[derive(Derivative, Display, Error)]
#[derivative(
    Clone(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub enum ReplaceError<S: Schema> {
    /// Inserted content deeper than insertion position
    InsertTooDeep,
    /// Inconsistent open depths
    InconsistentOpenDepths {
        /// Depth at the start
        from_depth: usize,
        /// How many nodes are "open" at the start
        open_start: usize,
        /// Depth at the end
        to_depth: usize,
        /// How many nodes are "open" at the end
        open_end: usize,
    },
    /// Could not resolve an index
    Resolve(#[from] ResolveErr),
    /// Cannot join {0:?} onto {1:?}
    CannotJoin(S::NodeType, S::NodeType),
    /// Invalid content for node {0:?}
    InvalidContent(S::NodeType),
}

pub(crate) fn replace<S: Schema>(
    rp_from: &ResolvedPos<S>,
    rp_to: &ResolvedPos<S>,
    slice: &Slice<S>, // FIXME: use Cow?
) -> Result<S::Node, ReplaceError<S>> {
    if slice.open_start > rp_from.depth {
        Err(ReplaceError::InsertTooDeep)
    } else if rp_from.depth - slice.open_start != rp_to.depth - slice.open_end {
        Err(ReplaceError::InconsistentOpenDepths {
            from_depth: rp_from.depth,
            open_start: slice.open_start,
            to_depth: rp_to.depth,
            open_end: slice.open_end,
        })
    } else {
        replace_outer(rp_from, rp_to, slice, 0)
    }
}

pub(crate) fn replace_outer<S: Schema>(
    rp_from: &ResolvedPos<S>,
    rp_to: &ResolvedPos<S>,
    slice: &Slice<S>,
    depth: usize,
) -> Result<S::Node, ReplaceError<S>> {
    let index = rp_from.index(depth);
    let node = rp_from.node(depth);
    if index == rp_to.index(depth) && depth < rp_from.depth - slice.open_start {
        // When both `from` and `to` are in the same child and the we are not at an open node yet
        let inner = replace_outer(rp_from, rp_to, slice, depth + 1)?;
        Ok(node.copy(|c| c.replace_child(index, inner).into_owned()))
    } else if slice.content.size() == 0 {
        // When we just delete content, i.e. the replacement slice is empty
        let content = replace_two_way(rp_from, &rp_to, depth)?;
        close(node, content)
    } else if slice.open_start == 0
        && slice.open_end == 0
        && rp_from.depth == depth
        && rp_to.depth == depth
    {
        // If not parent nodes are open, and the position of both `from` and `to` is at
        // the current level:

        // Simple, flat case
        let parent = rp_from.parent();
        let content = parent.content().unwrap_or(Fragment::EMPTY_REF);

        let new_content = content
            .cut(0..rp_from.parent_offset)
            .append(slice.content.clone())
            .append(content.cut(rp_to.parent_offset..));
        close(parent, new_content)
    } else {
        let (n, start, end) = prepare_slice_for_replace(slice, &rp_from);
        let rp_start = n.resolve(start)?;
        let rp_end = n.resolve(end)?;
        let content = replace_three_way(rp_from, &rp_start, &rp_end, &rp_to, depth)?;
        close(node, content)
    }
}

fn check_join<S: Schema>(main: &S::Node, sub: &S::Node) -> Result<(), ReplaceError<S>> {
    let sub_type = sub.r#type();
    let main_type = main.r#type();
    if sub_type.compatible_content(main_type) {
        Ok(())
    } else {
        Err(ReplaceError::CannotJoin(sub_type, main_type))
    }
}

fn joinable<'a, S: Schema>(
    rp_before: &ResolvedPos<'a, S>,
    rp_after: &ResolvedPos<'a, S>,
    depth: usize,
) -> Result<&'a S::Node, ReplaceError<S>> {
    let node = rp_before.node(depth);
    check_join::<S>(node, rp_after.node(depth))?;
    Ok(node)
}

fn add_node<S: Schema>(child: Cow<S::Node>, target: &mut Vec<S::Node>) {
    if let Some(last) = target.last_mut() {
        if let Some(c_text) = child.text_node() {
            if let Some(l_text) = c_text.same_markup(last) {
                let new_text_node = c_text.with_text(l_text.text.join(&c_text.text));
                *last = S::Node::from(new_text_node);
                return;
            }
        }
    }
    target.push(child.into_owned());
}

type Range<'b, 'a, S> = EitherOrBoth<&'b ResolvedPos<'a, S>, &'b ResolvedPos<'a, S>>;

fn add_range<S: Schema>(range: Range<S>, depth: usize, target: &mut Vec<S::Node>) {
    let node = range.right_or_left().node(depth);
    let mut start_index = 0;

    let end_index = if let Some(rp_end) = range.right() {
        rp_end.index(depth)
    } else {
        node.child_count()
    };

    if let Some(rp_start) = range.left() {
        start_index = rp_start.index(depth);
        if rp_start.depth > depth {
            start_index += 1;
        } else if rp_start.text_offset() > 0 {
            add_node::<S>(rp_start.node_after().unwrap(), target);
            start_index += 1;
        }
    }
    for i in start_index..end_index {
        add_node::<S>(Cow::Borrowed(node.child(i).unwrap()), target);
    }
    if let Some(rp_end) = range.right() {
        if rp_end.depth == depth && rp_end.text_offset() > 0 {
            add_node::<S>(rp_end.node_before().unwrap(), target);
        }
    }
}

fn close<S: Schema>(node: &S::Node, content: Fragment<S>) -> Result<S::Node, ReplaceError<S>> {
    let node_type = node.r#type();
    if node_type.valid_content(&content) {
        Ok(node.copy(|_| content))
    } else {
        Err(ReplaceError::InvalidContent(node_type))
    }
}

fn replace_three_way<S: Schema>(
    rp_from: &ResolvedPos<S>,
    rp_start: &ResolvedPos<S>,
    rp_end: &ResolvedPos<S>,
    rp_to: &ResolvedPos<S>,
    depth: usize,
) -> Result<Fragment<S>, ReplaceError<S>> {
    let open_start = if rp_from.depth > depth {
        Some(joinable(&rp_from, &rp_start, depth + 1)?)
    } else {
        None
    };
    let open_end = if rp_to.depth > depth {
        Some(joinable(&rp_end, rp_to, depth + 1)?)
    } else {
        None
    };

    let mut content = Vec::new();
    add_range(Range::Right(&rp_from), depth, &mut content);
    match (open_start, open_end) {
        (Some(os), Some(oe)) if rp_start.index(depth) == rp_end.index(depth) => {
            check_join(os, oe)?;
            let inner = replace_three_way(rp_from, rp_start, rp_end, rp_to, depth + 1)?;
            let closed = close(os, inner)?;
            add_node::<S>(Cow::Owned(closed), &mut content)
        }
        _ => {
            if let Some(os) = open_start {
                let inner = replace_two_way(rp_from, &rp_start, depth + 1)?;
                let closed = close(os, inner)?;
                add_node::<S>(Cow::Owned(closed), &mut content);
            }
            add_range(Range::Both(&rp_start, &rp_end), depth, &mut content);
            if let Some(oe) = open_end {
                let inner = replace_two_way(rp_end, rp_to, depth + 1)?;
                let closed = close(oe, inner)?;
                add_node::<S>(Cow::Owned(closed), &mut content);
            }
        }
    }
    add_range(Range::Left(rp_to), depth, &mut content);
    Ok(Fragment::from(content))
}

fn replace_two_way<S: Schema>(
    rp_from: &ResolvedPos<S>,
    rp_to: &ResolvedPos<S>,
    depth: usize,
) -> Result<Fragment<S>, ReplaceError<S>> {
    let mut content = Vec::new();
    add_range(Range::Right(rp_from), depth, &mut content);
    if rp_from.depth > depth {
        let r#type = joinable(rp_from, rp_to, depth + 1)?;
        let inner = replace_two_way(rp_from, rp_to, depth + 1)?;
        let child = close(r#type, inner)?;
        add_node::<S>(Cow::Owned(child), &mut content);
    }
    add_range(Range::Left(rp_to), depth, &mut content);
    Ok(Fragment::from(content))
}

fn prepare_slice_for_replace<'a, S: Schema>(
    slice: &'a Slice<S>,
    rp_along: &ResolvedPos<'a, S>,
) -> (S::Node, usize, usize) {
    let extra = rp_along.depth - slice.open_start;
    let parent = rp_along.node(extra);
    let mut node = parent.copy(|_| slice.content.clone());
    for i in (0..extra).rev() {
        node = rp_along.node(i).copy(|_| Fragment::from((node,)));
    }

    let start = slice.open_start + extra;
    let end = node.content_size() - slice.open_end - extra;
    (node, start, end)
}