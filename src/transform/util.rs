use serde::{Deserialize, Serialize};

/// A span within a document
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Span {
    /// Start of the span
    pub from: usize,
    /// End of the span
    pub to: usize,
}
