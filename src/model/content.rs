use crate::model::{Fragment, Schema};
use displaydoc::Display;
use std::ops::RangeBounds;
use thiserror::Error;

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

    /// Try to match a part of a fragment. Returns the resulting match when successful.
    fn match_fragment_range<R: RangeBounds<usize>>(
        self,
        fragment: &Fragment<S>,
        range: R,
    ) -> Option<Self>;

    /// True when this match state represents a valid end of the node.
    fn valid_end(self) -> bool;

    /// Match a node type, returning a match after that node if successful.
    fn match_type(self, r#type: S::NodeType) -> Option<Self>;
}
