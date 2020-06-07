use crate::model::Fragment;
use serde::{Deserialize, Serialize};

/// A slice of a fragment
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Slice {
    content: Fragment,
    open_start: Option<usize>,
    open_end: Option<usize>,
}
