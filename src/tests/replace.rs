use prosemirror_markdown::{
    helper::{blockquote, doc, h1, li, p, ul},
    MarkdownNode, MarkdownNodeType, MD,
};
use prosemirror_model::{Fragment, Node, Slice, SliceError, ReplaceError};
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
    let e = ReplaceError::CannotJoin(MarkdownNodeType::BulletList, MarkdownNodeType::Blockquote);

    bad::<_, Range<usize>>((t, 4..6), None, e);
}

#[test]
fn check_content_validity() {
    let t = doc(blockquote(p("hi"))); // 1..6
    let i = doc(blockquote("hi")); // 3..4
    let e = ReplaceError::InvalidContent(MarkdownNodeType::Blockquote);

    bad((t, 1..6), Some((i, 3..4)), e);
}
