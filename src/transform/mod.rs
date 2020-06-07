//! # The document transformations
//!
mod util;

pub use util::Span;

use crate::model::{Schema, Slice};
use serde::{Deserialize, Serialize};

/// A list of steps
#[allow(type_alias_bounds)]
pub type Steps<S: Schema> = Vec<Step<S>>;

/// Steps that can be applied on a document
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "stepType", rename_all = "camelCase")]
pub enum Step<S: Schema> {
    /// Replace some content
    Replace(ReplaceStep<S>),
    /// Replace around some content
    ReplaceAround(ReplaceAroundStep<S>),
    /// Add a mark to a span
    AddMark(AddMarkStep<S>),
    /// Remove a mark from a span
    RemoveMark(RemoveMarkStep<S>),
}

/// Replace some part of the document
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceStep<S: Schema> {
    /// The affected span
    #[serde(flatten)]
    pub span: Span,
    /// The slice to replace the current content with
    pub slice: Option<Slice<S>>,
    /// Whether this is a structural change
    pub structure: Option<bool>,
}

/// Replace the document structure while keeping some content
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceAroundStep<S: Schema> {
    /// The affected part of the document
    #[serde(flatten)]
    pub span: Span,
    /// Start of the gap
    pub gap_from: usize,
    /// End of the gap
    pub gap_to: usize,
    /// The inner slice
    pub slice: Option<Slice<S>>,
    /// ???
    pub insert: usize,
    /// ???
    pub structure: Option<bool>,
}

/// Adding a mark on some part of the document
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AddMarkStep<S: Schema> {
    /// The affected part of the document
    #[serde(flatten)]
    pub span: Span,
    /// The mark to add
    pub mark: S::Mark,
}

/// Removing a mark on some part of the document
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMarkStep<S: Schema> {
    /// The affected part of the document
    #[serde(flatten)]
    pub span: Span,
    /// The mark to remove
    pub mark: S::Mark,
}

#[cfg(test)]
mod tests {
    use super::{AddMarkStep, ReplaceStep, Span, Step};
    use crate::markdown::{MarkdownMark, MarkdownNode, MarkdownSchema as Schema};
    use crate::model::{Fragment, Node, Slice};

    #[test]
    fn test_deserialize() {
        let s1: Step<Schema> = serde_json::from_str(
            r#"{"stepType":"addMark","mark":{"type":"em"},"from":61,"to":648}"#,
        )
        .unwrap();

        assert_eq!(
            s1,
            Step::AddMark(AddMarkStep {
                span: Span { from: 61, to: 648 },
                mark: MarkdownMark::Em,
            })
        );

        let s2: Step<Schema> = serde_json::from_str(
            r#"{"stepType":"replace","from":986,"to":986,"slice":{"content":[{"type":"text","text":"!"}]}}"#
        ).unwrap();

        assert_eq!(
            s2,
            Step::Replace(ReplaceStep {
                span: Span { from: 986, to: 986 },
                slice: Some(Slice {
                    content: Fragment::from((MarkdownNode::text("!"),)),
                    open_start: None,
                    open_end: None,
                }),
                structure: None,
            })
        );
    }
}
