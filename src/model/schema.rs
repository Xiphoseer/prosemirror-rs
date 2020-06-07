use super::{Mark, Node};

/// This type represents a schema.
pub trait Schema: Clone {
    /// This type represents any of the marks that are valid in the schema.
    type Mark: Mark;
    /// This type represents any of the nodes that are valid in the schema.
    type Node: Node;
}
