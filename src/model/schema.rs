use super::{Fragment, Mark, MarkSet, Node, Text};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// This type represents a schema.
pub trait Schema: Sized + 'static {
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

impl<S: Schema> Block<S> {
    /// Copies this block, mapping the content
    pub fn copy<F>(&self, map: F) -> Self
    where
        F: FnOnce(&Fragment<S>) -> Fragment<S>,
    {
        Block {
            content: map(&self.content),
        }
    }
}

/// A node with attributes
#[derive(Derivative, Deserialize, Serialize)]
#[derivative(
    Debug(bound = "A: Debug"),
    Clone(bound = "A: Clone"),
    Default(bound = "A: Default"),
    PartialEq(bound = "A: PartialEq"),
    Eq(bound = "A: Eq")
)]
#[serde(bound = "A: for<'d> Deserialize<'d> + Serialize")]
pub struct AttrNode<S: Schema, A> {
    /// Attributes
    pub attrs: A,

    /// The content.
    #[serde(default)]
    #[derivative(Debug(bound = ""))]
    pub content: Fragment<S>,
}

impl<S: Schema, A: Clone> AttrNode<S, A> {
    /// Copies this block, mapping the content
    pub fn copy<F>(&self, map: F) -> Self
    where
        F: FnOnce(&Fragment<S>) -> Fragment<S>,
    {
        AttrNode {
            content: map(&self.content),
            attrs: self.attrs.clone(),
        }
    }
}

/// A text node
#[derive(Derivative, Deserialize, Serialize)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Default(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
#[serde(bound = "")]
pub struct TextNode<S: Schema> {
    // todo: replace with typemap
    /// Marks on this node
    #[serde(default)]
    pub marks: MarkSet<S>,
    /// The actual text
    pub text: Text,
}

/// A leaf node (just attributes)
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Leaf<A> {
    /// Attributes
    pub attrs: A,
}
