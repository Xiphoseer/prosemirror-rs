use crate::model::Fragment;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Slice {
    content: Fragment,
    open_start: Option<usize>,
    open_end: Option<usize>,
}
