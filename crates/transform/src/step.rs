use prosemirror_model::{InsertError, ReplaceError, ResolveErr, Schema, SliceError};
use derivative::Derivative;
use displaydoc::Display;
use thiserror::Error;

/// Different ways a step application can fail
#[derive(Derivative, Display, Error)]
#[derivative(Debug(bound = ""))]
pub enum StepError<S: Schema> {
    /// Structure replace would overwrite content
    WouldOverwrite,
    /// Structure gap-replace would overwrite content
    GapWouldOverwrite,
    /// Gap is not a flat range
    GapNotFlat,
    /// Content does not fit in gap
    GapNotFit,
    /// Invalid indices
    Resolve(#[from] ResolveErr),
    /// Invalid resolve
    Replace(#[from] ReplaceError<S>),
    /// Invalid slice
    Slice(#[from] SliceError),
    /// Insert error
    Insert(#[from] InsertError),
}

/// The result of [applying](#transform.Step.apply) a step. Contains either a
/// new document or a failure value.
#[allow(type_alias_bounds)]
pub type StepResult<S: Schema> = Result<S::Node, StepError<S>>;

/// A step object represents an atomic change.
///
/// It generally applies only to the document it was created for, since the positions
/// stored in it will only make sense for that document.
pub trait StepKind<S: Schema> {
    /// Applies this step to the given document, returning a result
    /// object that either indicates failure, if the step can not be
    /// applied to this document, or indicates success by containing a
    /// transformed document.
    fn apply(&self, doc: &S::Node) -> StepResult<S>;
}
