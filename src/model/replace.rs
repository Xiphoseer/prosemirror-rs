use super::{fragment::IndexError, Index};
use crate::model::{ContentMatchError, Fragment, Node, NodeType, ResolveErr, ResolvedPos, Schema};
use crate::util::EitherOrBoth;
use derivative::Derivative;
use displaydoc::Display;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use thiserror::Error;

/// A slice of a fragment
#[derive(Derivative, Deserialize, Serialize)]
#[derivative(
    Debug(bound = ""),
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

    pub(crate) fn insert_at(
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

#[cfg(test)]
mod tests {
    use super::ReplaceError;
    use crate::markdown::helper::{blockquote, doc, h1, li, p, ul};
    use crate::markdown::{MarkdownNode, MarkdownNodeType, MD};
    use crate::model::{Fragment, Node, Slice, SliceError};
    use displaydoc::Display;
    use std::fmt::Debug;
    use std::ops::{Range, RangeBounds};
    use thiserror::Error;

    #[derive(Debug, Display, Error, PartialEq, Eq)]
    enum RplError {
        /// Could not slice
        Slice(#[from] SliceError),
    }

    // insert.slice(insert.tag.a, insert.tag.b)
    fn rpl<DR: RangeBounds<usize> + Debug, SR: RangeBounds<usize> + Debug>(
        (doc, range): (MarkdownNode, DR),
        insert: Option<(MarkdownNode, SR)>,
        expected: MarkdownNode,
    ) -> Result<(), RplError> {
        let slice = insert
            .map(|(n, r)| n.slice(r, false).unwrap())
            .unwrap_or_default();

        assert_eq!(doc.replace(range, &slice), Ok(expected));
        Ok(())
    }

    #[test]
    fn join_on_delete() {
        let t1: MarkdownNode = doc((p("one"), p("two")));
        let e1: MarkdownNode = doc((p("onwo"),));
        rpl::<_, Range<usize>>((t1, 3..7), None, e1).unwrap();
    }

    #[test]
    fn merges_matching_blocks() {
        let t2: MarkdownNode = doc((p("one"), p("two")));
        let i2: MarkdownNode = doc((p("xxxx"), p("yyyy")));

        let f2: Fragment<MD> = Fragment::from(vec![p("xx"), p("yy")]);
        assert_eq!(i2.slice(3..9, false), Ok(Slice::new(f2, 1, 1)));

        let e2: MarkdownNode = doc((p("onxx"), p("yywo")));
        rpl((t2, 3..7), Some((i2, 3..9)), e2).unwrap();
    }

    #[test]
    fn merges_when_adding_text() {
        let (t3, r3) = (doc((p("one"), p("two"))), 3..7);
        let (i3, s3) = (doc((p("H"),)), 1..2);
        let e3 = doc((p("onHwo"),));

        rpl((t3, r3), Some((i3, s3)), e3).unwrap();
    }

    #[test]
    fn can_insert_text() {
        let t4 = doc(vec![p("before"), p("one"), p("after")]);
        let r4 = 11..11;

        let i4 = doc(vec![p("H")]);
        let s4 = 1..2;

        let e4 = doc(vec![p("before"), p("onHe"), p("after")]);
        rpl((t4, r4), Some((i4, s4)), e4).unwrap();
    }

    #[test]
    fn doesnt_merge_non_matching_blocks() {
        // What does this test? The name seems to contradict that it succeeds
        // This is from the original prosemirror test cases

        let t5 = doc(vec![p("one"), p("two")]);
        let r5 = 3..7;

        let i5 = doc(vec![h1("H")]);
        let s5 = 1..2;

        let e5 = doc(vec![p("onHwo")]);
        rpl((t5, r5), Some((i5, s5)), e5).unwrap();
    }

    #[test]
    fn can_merge_a_nested_node() {
        let t6 = doc(blockquote(blockquote(vec![p("one"), p("two")])));
        let i6 = doc(p("H"));
        let e6 = doc(blockquote(blockquote(p("onHwo"))));

        rpl((t6, 5..9), Some((i6, 1..2)), e6).unwrap();
    }

    #[test]
    fn can_replace_within_a_block() {
        let t = doc(blockquote(p("abcd")));
        let i = doc(p("xyz"));
        let e = doc(blockquote(p("ayd")));

        rpl((t, 3..5), Some((i, 2..3)), e).unwrap();
    }

    #[test]
    fn can_insert_a_lopsided_slice() {
        let t = doc(blockquote(blockquote(vec![p("one"), p("two"), p("three")]))); //5..12
        let i = doc(blockquote(vec![p("aaaa"), p("bb"), p("cc"), p("dd")])); //4..15
        let e = doc(blockquote(blockquote(vec![
            p("onaa"),
            p("bb"),
            p("cc"),
            p("three"),
        ])));

        rpl((t, 5..12), Some((i, 4..15)), e).unwrap();
    }

    #[test]
    fn can_insert_a_deep_lopsided_slice() {
        let t = doc(blockquote(vec![
            blockquote(vec![p("one"), p("two"), p("three")]),
            p("x"),
        ])); //5..20
        let i = doc(vec![blockquote(vec![p("aaaa"), p("bb"), p("cc")]), p("dd")]); // 4..16
        let e = doc(blockquote(vec![
            blockquote(vec![p("onaa"), p("bb"), p("cc")]),
            p("x"),
        ]));

        rpl((t, 5..20), Some((i, 4..16)), e).unwrap();
    }

    #[test]
    fn can_merge_multiple_levels() {
        let t = doc(vec![
            blockquote(blockquote(p("hello"))),
            blockquote(blockquote(p("a"))),
        ]);
        let e = doc(blockquote(blockquote(p("hella"))));

        rpl::<_, Range<usize>>((t, 7..14), None, e).unwrap();
    }

    #[test]
    fn can_merge_multiple_levels_while_inserting() {
        let t = doc(vec![
            blockquote(blockquote(p("hello"))),
            blockquote(blockquote(p("a"))),
        ]);
        let i = doc(p("i"));
        let e = doc(blockquote(blockquote(p("hellia"))));

        rpl((t, 7..14), Some((i, 1..2)), e).unwrap();
    }

    #[test]
    fn can_insert_a_split() {
        let t = doc(p("foobar")); // 4..4
        let i = doc(vec![p("x"), p("y")]); // 1..5
        let e = doc(vec![p("foox"), p("ybar")]);

        rpl((t, 4..4), Some((i, 1..5)), e).unwrap();
    }

    #[test]
    fn can_insert_a_deep_split() {
        let t = doc(blockquote(p("fooxbar")));
        let i = doc(vec![blockquote(p("x")), blockquote(p("y"))]);
        let e = doc(vec![blockquote(p("foox")), blockquote(p("ybar"))]);

        rpl((t, 5..6), Some((i, 2..8)), e).unwrap();
    }

    #[test]
    fn can_add_a_split_one_level_up() {
        let t = doc(blockquote(vec![p("foou"), p("vbar")]));
        let i = doc(vec![blockquote(p("x")), blockquote(p("y"))]);
        let e = doc(vec![blockquote(p("foox")), blockquote(p("ybar"))]);

        rpl((t, 5..9), Some((i, 2..8)), e).unwrap();
    }

    #[test]
    fn keeps_the_node_type_of_the_left_node() {
        let t = doc(h1("foobar"));
        let i = doc(p("foobaz"));
        let e = doc(h1("foobaz"));

        rpl((t, 4..8), Some((i, 4..8)), e).unwrap();
    }

    #[test]
    fn keeps_the_node_type_even_when_empty() {
        let t = doc(h1("bar"));
        let i = doc(p("foobaz"));
        let e = doc(h1("baz"));

        rpl((t, 1..5), Some((i, 4..8)), e).unwrap();
    }

    fn bad<DR: RangeBounds<usize> + Debug, SR: RangeBounds<usize> + Debug>(
        (doc, range): (MarkdownNode, DR),
        insert: Option<(MarkdownNode, SR)>,
        pattern: ReplaceError<MD>,
    ) {
        let slice = insert
            .map(|(n, r)| n.slice(r, false).unwrap())
            .unwrap_or_default();
        assert_eq!(doc.replace(range, &slice), Err(pattern));
    }

    #[test]
    fn doesnt_allow_the_left_side_to_be_too_deep() {
        let t = doc(p("")); // 1..1
        let i = doc(blockquote(p(""))); // 2..4
        bad((t, 1..1), Some((i, 2..4)), ReplaceError::InsertTooDeep);
    }

    #[test]
    fn doesnt_allow_a_depth_mismatch() {
        let t = doc(p("")); // 1..1
        let i = doc(p("")); // 0..1
        bad(
            (t, 1..1),
            Some((i, 0..1)),
            ReplaceError::InconsistentOpenDepths {
                from_depth: 1,
                open_start: 0,
                to_depth: 1,
                open_end: 1,
            },
        );
    }

    #[test]
    fn rejects_a_bad_fit() {
        let t = doc(vec![]); // 0..0
        let i = doc(p("foo")); // 1..4
        let e = ReplaceError::InvalidContent(MarkdownNodeType::Doc);

        bad((t, 0..0), Some((i, 1..4)), e);
    }

    #[test]
    fn rejects_unjoinable_content() {
        let t = doc(ul(li(p("a")))); // 6..7
        let i = doc(p("foo")); //4..5
        let e = ReplaceError::CannotJoin(MarkdownNodeType::Paragraph, MarkdownNodeType::BulletList);

        bad((t, 6..7), Some((i, 4..5)), e);
    }

    #[test]
    fn rejects_an_unjoinable_delete() {
        let t = doc(vec![blockquote(p("a")), ul(li(p("b")))]); //4..6
        let e =
            ReplaceError::CannotJoin(MarkdownNodeType::BulletList, MarkdownNodeType::Blockquote);

        bad::<_, Range<usize>>((t, 4..6), None, e);
    }

    #[test]
    fn check_content_validity() {
        let t = doc(blockquote(p("hi"))); // 1..6
        let i = doc(blockquote("hi")); // 3..4
        let e = ReplaceError::InvalidContent(MarkdownNodeType::Blockquote);

        bad((t, 1..6), Some((i, 3..4)), e);
    }
}
