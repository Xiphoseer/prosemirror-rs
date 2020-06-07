use crate::model::{Slice, Mark};
use serde::{Deserialize, Serialize};

pub type Steps = Vec<Step>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "stepType", rename_all = "camelCase")]
pub enum Step {
    #[serde(rename_all = "camelCase")]
    Replace {
        from: usize,
        to: usize,
        slice: Option<Slice>,
        structure: Option<bool>,
    },
    #[serde(rename_all = "camelCase")]
    ReplaceAround {
        from: usize,
        to: usize,
        gap_from: usize,
        gap_to: usize,
        slice: Option<Slice>,
        insert: usize,
        structure: Option<bool>,
    },
    #[serde(rename_all = "camelCase")]
    AddMark { from: usize, to: usize, mark: Mark },
    #[serde(rename_all = "camelCase")]
    RemoveMark { from: usize, to: usize, mark: Mark },
}
