use crate::{Fragment, Schema};
use displaydoc::Display;
use std::ops::RangeBounds;
use thiserror::Error;

use crate::node::Node;
use crate::util;

/// Error on content matching
#[derive(Debug, Display, Error)]
pub enum ContentMatchError {
    /// Called contentMatchAt on a node with invalid content
    InvalidContent,
}

/// Instances of this class represent a match state of a node type's content expression, and can be
/// used to find out whether further content matches here, and whether a given position is a valid end of the node.
pub trait ContentMatch<S: Schema>: Copy {
    /// Try to match a fragment. Returns the resulting match when successful.
    fn match_fragment(self, fragment: &Fragment<S>) -> Option<Self> {
        self.match_fragment_range(fragment, ..)
    }

    /// True when this match state represents a valid end of the node.
    fn valid_end(self) -> bool;

    /// Match a node type, returning a match after that node if successful.
    fn match_type(self, r#type: S::NodeType) -> Option<Self>;
}

pub trait ContentMatchExt<S: Schema>: ContentMatch<S> {
    /// Try to match a part of a fragment. Returns the resulting match when successful.
    fn match_fragment_range<R: RangeBounds<usize>>(
        self,
        fragment: &Fragment<S>,
        range: R,
    ) -> Option<Self>;
}

impl<T, S: Schema> ContentMatchExt<S> for T
where
    T: ContentMatch<S>,
{
    fn match_fragment_range<R: RangeBounds<usize>>(
        self,
        fragment: &Fragment<S>,
        range: R,
    ) -> Option<Self> {
        let start = util::from(&range);
        let end = util::to(&range, fragment.child_count());

        let mut test = self;
        for child in &fragment.children()[start..end] {
            match test.match_type(child.r#type()) {
                Some(next) => {
                    test = next;
                }
                None => {
                    return None;
                }
            }
        }
        Some(test)
    }
}
