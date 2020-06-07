use super::{Fragment, Mark, Node};
use serde::{Deserialize, Serialize};

/// This type represents a schema.
pub trait Schema: Clone + 'static {
    /// This type represents any of the marks that are valid in the schema.
    type Mark: Mark;
    /// This type represents any of the nodes that are valid in the schema.
    type Node: Node<Self>;
}

/// A simple block node
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Block<S: Schema> {
    /// The content.
    #[serde(default)]
    pub content: Fragment<S>,
}
