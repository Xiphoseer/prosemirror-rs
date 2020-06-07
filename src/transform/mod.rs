//! # The document transformations
//!
mod util;

pub use util::Span;

use crate::model::{Mark, Slice};
use serde::{Deserialize, Serialize};

/// A list of steps
pub type Steps = Vec<Step>;

/// Steps that can be applied on a document
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "stepType", rename_all = "camelCase")]
pub enum Step {
    /// Replace some content
    Replace(ReplaceStep),
    /// Replace around some content
    ReplaceAround(ReplaceAroundStep),
    /// Add a mark to a span
    AddMark(AddMarkStep),
    /// Remove a mark from a span
    RemoveMark(RemoveMarkStep),
}

/// Replace some part of the document
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceStep {
    /// The affected span
    #[serde(flatten)]
    span: Span,
    /// The slice to replace the current content with
    slice: Option<Slice>,
    /// Whether this is a structural change
    structure: Option<bool>,
}

/// Replace the document structure while keeping some content
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceAroundStep {
    /// The affected part of the document
    #[serde(flatten)]
    span: Span,
    /// Start of the gap
    gap_from: usize,
    /// End of the gap
    gap_to: usize,
    /// The inner slice
    slice: Option<Slice>,
    /// ???
    insert: usize,
    /// ???
    structure: Option<bool>,
}

/// Adding a mark on some part of the document
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AddMarkStep {
    /// The affected part of the document
    #[serde(flatten)]
    span: Span,
    /// The mark to add
    mark: Mark,
}

/// Removing a mark on some part of the document
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMarkStep {
    /// The affected part of the document
    #[serde(flatten)]
    span: Span,
    /// The mark to remove
    mark: Mark,
}

#[cfg(test)]
mod tests {
    use super::{AddMarkStep, Span, Step};
    use crate::model::Mark;

    #[test]
    fn test_deserialize() {
        let s1: Vec<Step> = serde_json::from_str(
            r#"[{"stepType":"addMark","mark":{"type":"em"},"from":61,"to":648}]"#,
        )
        .unwrap();

        assert_eq!(
            s1,
            vec![Step::AddMark(AddMarkStep {
                span: Span { from: 61, to: 648 },
                mark: Mark::Em,
            })]
        );
    }
}
