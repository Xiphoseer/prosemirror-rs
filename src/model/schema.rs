use super::{Fragment, Mark, Node};
use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// This type represents a schema.
pub trait Schema: Clone + 'static {
    /// This type represents any of the marks that are valid in the schema.
    type Mark: Mark;
    /// This type represents any of the nodes that are valid in the schema.
    type Node: Node<Self>;
}

/// A simple block node
#[derive(Derivative, Deserialize, Serialize)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Default(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
#[serde(bound = "")]
pub struct Block<S: Schema> {
    /// The content.
    #[serde(default)]
    #[derivative(Debug(bound = ""))]
    pub content: Fragment<S>,
}
