use super::de;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub type MarkSet = HashSet<Mark>;

#[derive(Debug, Hash, Eq, Clone, PartialEq, Deserialize, Serialize)]
pub struct LinkAttrs {
    href: String,
    #[serde(default, deserialize_with = "de::deserialize_or_default")]
    title: String,
}

#[derive(Debug, Hash, Eq, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Mark {
    Strong,
    Em,
    Code,
    Link { attrs: LinkAttrs },
}
