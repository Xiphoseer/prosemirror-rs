//! # The document transformations
//!
mod mark_step;
mod replace_step;
mod step;
mod util;

pub use mark_step::{AddMarkStep, RemoveMarkStep};
pub use replace_step::{ReplaceAroundStep, ReplaceStep};
pub use step::{StepError, StepKind, StepResult};
pub use util::Span;

use crate::model::Schema;
use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// A list of steps
#[allow(type_alias_bounds)]
pub type Steps<S: Schema> = Vec<Step<S>>;

/// Steps that can be applied on a document
#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug(bound = ""), PartialEq(bound = ""), Eq(bound = ""))]
#[serde(bound = "", tag = "stepType", rename_all = "camelCase")]
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

#[cfg(test)]
mod tests {
    use super::{AddMarkStep, ReplaceStep, Span, Step};
    use crate::markdown::{MarkdownMark, MarkdownNode, MD};
    use crate::model::{Fragment, Node, Slice};

    #[test]
    fn test_deserialize() {
        let s1: Step<MD> = serde_json::from_str(
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

        let s2: Step<MD> = serde_json::from_str(
            r#"{"stepType":"replace","from":986,"to":986,"slice":{"content":[{"type":"text","text":"!"}]}}"#
        ).unwrap();

        assert_eq!(
            s2,
            Step::Replace(ReplaceStep {
                span: Span { from: 986, to: 986 },
                slice: Slice {
                    content: Fragment::from((MarkdownNode::text("!"),)),
                    open_start: 0,
                    open_end: 0,
                },
                structure: false,
            })
        );
    }
}
