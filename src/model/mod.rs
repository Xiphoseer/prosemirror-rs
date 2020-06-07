//! # The document model
//!
//! This module is derived from the `prosemirror-markdown` schema and the
//! the general JSON serialization of nodes.
mod fragment;
mod marks;
mod node;
mod resolved_pos;
mod schema;
mod slice;
pub(crate) mod util;

pub use fragment::Fragment;
pub use marks::{Mark, MarkSet};
pub use node::{Node, Text};
pub(crate) use resolved_pos::Index;
pub use resolved_pos::{ResolveErr, ResolvedPos};
pub use schema::{Block, Schema};
pub use slice::Slice;

#[cfg(test)]
mod tests {
    use super::{Index, Node, ResolvedPos};
    use crate::markdown::{helper::*, ImageAttrs, MarkdownNode, MarkdownSchema as Schema};
    use std::ops::Deref;

    #[test]
    fn test_null_string() {
        assert_eq!(
            serde_json::from_str::<ImageAttrs>(r#"{"src": "", "alt": null}"#).unwrap(),
            ImageAttrs {
                src: String::new(),
                title: String::new(),
                alt: String::new()
            }
        );
    }

    #[test]
    fn test_deserialize_text() {
        assert_eq!(
            serde_json::from_str::<MarkdownNode>(r#"{"type": "text", "text": "Foo"}"#).unwrap(),
            MarkdownNode::text("Foo"),
        );
    }

    #[test]
    fn test_size() {
        assert_eq!(node("Hello").node_size(), 5);
        assert_eq!(node("\u{1F60A}").node_size(), 2);

        let test_3 = p(("Hallo", "Foo"));
        assert_eq!(test_3.node_size(), 10);
        let ct_3 = test_3.content().unwrap();
        assert_eq!(ct_3.find_index(0, false), Ok(Index::new(0, 0)));
        assert_eq!(ct_3.find_index(1, false), Ok(Index::new(0, 0)));
        assert_eq!(ct_3.find_index(2, false), Ok(Index::new(0, 0)));
        assert_eq!(ct_3.find_index(3, false), Ok(Index::new(0, 0)));
        assert_eq!(ct_3.find_index(4, false), Ok(Index::new(0, 0)));
        assert_eq!(ct_3.find_index(5, false), Ok(Index::new(1, 5)));
        assert_eq!(ct_3.find_index(6, false), Ok(Index::new(1, 5)));
        assert_eq!(ct_3.find_index(7, false), Ok(Index::new(1, 5)));
        assert_eq!(ct_3.find_index(8, false), Ok(Index::new(2, 8)));
        assert_eq!(ct_3.find_index(9, false), Err(()));

        assert_eq!(
            ResolvedPos::<Schema>::resolve(&test_3, 0),
            Ok(ResolvedPos::new(0, vec![(&test_3, 0, 0)], 0))
        );
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    struct Sol<'a> {
        node: &'a MarkdownNode,
        start: usize,
        end: usize,
    }

    fn sol(node: &MarkdownNode, start: usize, end: usize) -> Sol {
        Sol { node, start, end }
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum Exp<'a> {
        Node(&'a MarkdownNode),
        Str(&'static str),
        Null,
    }

    impl<'a> PartialEq<Exp<'a>> for Option<std::borrow::Cow<'a, MarkdownNode>> {
        fn eq(&self, other: &Exp<'a>) -> bool {
            if let Some(node) = self {
                match other {
                    Exp::Node(exp_node) => node.deref() == *exp_node,
                    Exp::Str(text) => &node.text_content() == text,
                    Exp::Null => false,
                }
            } else {
                *other == Exp::Null
            }
        }
    }

    #[test]
    fn test_resolve() {
        let test_doc = doc((p(("ab",)), blockquote((p((em("cd"), "ef")),))));
        let _doc = sol(&test_doc, 0, 12);
        let _p1 = sol(test_doc.child(0).unwrap(), 1, 3);
        let _blk = sol(test_doc.child(1).unwrap(), 5, 11);
        let _p2 = sol(_blk.node.child(0).unwrap(), 6, 10);

        let expected = [
            (&[_doc][..], 0, Exp::Null, Exp::Node(_p1.node)),
            (&[_doc, _p1], 0, Exp::Null, Exp::Str("ab")),
            (&[_doc, _p1], 1, Exp::Str("a"), Exp::Str("b")),
            (&[_doc, _p1], 2, Exp::Str("ab"), Exp::Null),
            (&[_doc], 4, Exp::Node(_p1.node), Exp::Node(_blk.node)),
            (&[_doc, _blk], 0, Exp::Null, Exp::Node(_p2.node)),
            (&[_doc, _blk, _p2], 0, Exp::Null, Exp::Str("cd")),
            (&[_doc, _blk, _p2], 1, Exp::Str("c"), Exp::Str("d")),
            (&[_doc, _blk, _p2], 2, Exp::Str("cd"), Exp::Str("ef")),
            (&[_doc, _blk, _p2], 3, Exp::Str("e"), Exp::Str("f")),
            (&[_doc, _blk, _p2], 4, Exp::Str("ef"), Exp::Null),
            (&[_doc, _blk], 6, Exp::Node(_p2.node), Exp::Null),
            (&[_doc], 12, Exp::Node(_blk.node), Exp::Null),
        ];

        for (pos, (path, parent_offset, before, after)) in expected.iter().enumerate() {
            let pos = ResolvedPos::<Schema>::resolve(&test_doc, pos).unwrap();
            assert_eq!(pos.depth, path.len() - 1);

            for (i, exp_i) in path.iter().enumerate() {
                let act = sol(pos.node(i), pos.start(i), pos.end(i));
                assert_eq!((i, &act), (i, exp_i));
                if i > 0 {
                    assert_eq!(pos.before(i), Some(exp_i.start - 1));
                    assert_eq!(pos.after(i), Some(exp_i.end + 1));
                }
            }
            assert_eq!(pos.parent_offset, *parent_offset);
            assert_eq!(pos.node_before(), *before);
            assert_eq!(pos.node_after(), *after);
        }
    }
}
