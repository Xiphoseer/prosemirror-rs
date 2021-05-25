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

use prosemirror_model::Schema;
use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// A list of steps
#[allow(type_alias_bounds)]
pub type Steps<S: Schema> = Vec<Step<S>>;

/// Steps that can be applied on a document
#[derive(Debug, Derivative, Deserialize, Serialize)]
#[derivative(PartialEq(bound = ""), Eq(bound = ""))]
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

impl<S: Schema> Step<S> {
    /// Apply the step to the given node
    pub fn apply(&self, doc: &S::Node) -> StepResult<S> {
        match self {
            Self::Replace(r_step) => r_step.apply(doc),
            Self::ReplaceAround(ra_step) => ra_step.apply(doc),
            Self::AddMark(am_step) => am_step.apply(doc),
            Self::RemoveMark(rm_step) => rm_step.apply(doc),
        }
    }
}