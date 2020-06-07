use crate::model::{Fragment, Schema};
use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// A slice of a fragment
#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug(bound = ""), PartialEq(bound = ""), Eq(bound = ""))]
#[serde(bound = "", rename_all = "camelCase")]
pub struct Slice<S: Schema> {
    /// The slice's content.
    pub content: Fragment<S>,
    /// The open depth at the start.
    pub open_start: Option<usize>,
    /// The open depth at the end.
    pub open_end: Option<usize>,
}
