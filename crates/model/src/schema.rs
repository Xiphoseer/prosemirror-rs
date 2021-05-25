use super::{ContentMatch, Fragment, Mark, MarkSet, Node, NodeType, Text};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// This type represents a schema.
pub trait Schema: Sized + 'static {
    /// This type represents any of the marks that are valid in the schema.
    type Mark: Mark<Self>;
    /// This type represents any of the mark types that are valid in the schema.
    type MarkType: MarkType;
    /// This type represents any of the nodes that are valid in the schema.
    type Node: Node<Self>;
    /// This type represents any of the node types that are valid in the schema.
    type NodeType: NodeType<Self>;
    /// This type represents the `ContentMatch` impl
    type ContentMatch: ContentMatch<Self>;
}

/// Implemented for model data containers
pub trait NodeImpl<S: Schema> {
    /// Copy the data using the mapping function for child content
    fn copy<F>(&self, map: F) -> Self
    where
        F: FnOnce(&Fragment<S>) -> Fragment<S>;

    /// Get the content of this node
    fn content(&self) -> Option<&Fragment<S>>;
}

/// A simple block node
#[derive(Debug, Derivative, Deserialize, Serialize)]
#[derivative(
    Default(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
#[serde(bound = "")]
pub struct Block<S: Schema> {
    /// The content.
    #[serde(default)]
    pub content: Fragment<S>,
}

impl<S: Schema> Clone for Block<S> {
    fn clone(&self) -> Self {
        Self { content: self.content.clone() }
    }
}

impl<S: Schema> NodeImpl<S> for Block<S> {
    /// Copies this block, mapping the content
    fn copy<F>(&self, map: F) -> Self
    where
        F: FnOnce(&Fragment<S>) -> Fragment<S>,
    {
        Block {
            content: map(&self.content),
        }
    }

    fn content(&self) -> Option<&Fragment<S>> {
        Some(&self.content)
    }
}

/// A node with attributes
#[derive(Debug, Derivative, Deserialize, Serialize)]
#[derivative(
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

impl<S: Schema, A: Clone> NodeImpl<S> for AttrNode<S, A> {
    /// Copies this block, mapping the content
    fn copy<F>(&self, map: F) -> Self
    where
        F: FnOnce(&Fragment<S>) -> Fragment<S>,
    {
        AttrNode {
            content: map(&self.content),
            attrs: self.attrs.clone(),
        }
    }

    fn content(&self) -> Option<&Fragment<S>> {
        Some(&self.content)
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

impl<S: Schema> NodeImpl<S> for TextNode<S> {
    fn copy<F>(&self, _: F) -> Self
    where
        F: FnOnce(&Fragment<S>) -> Fragment<S>,
    {
        self.clone()
    }

    fn content(&self) -> Option<&Fragment<S>> {
        None
    }
}

impl<S: Schema> TextNode<S> {
    /// Check whether the marks are identical
    pub fn same_markup<'o>(&self, other: &'o S::Node) -> Option<&'o TextNode<S>> {
        other.text_node().filter(|x| x.marks == self.marks)
    }

    /// Create a new `TextNode` with the given text
    pub fn with_text(&self, text: Text) -> Self {
        TextNode {
            marks: self.marks.clone(),
            text,
        }
    }
}

/// A leaf node (just attributes)
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Leaf<A> {
    /// Attributes
    pub attrs: A,
}

impl<S: Schema, A: Clone> NodeImpl<S> for Leaf<A> {
    /// Copies this block, mapping the content
    fn copy<F>(&self, _: F) -> Self
    where
        F: FnOnce(&Fragment<S>) -> Fragment<S>,
    {
        self.clone()
    }

    fn content(&self) -> Option<&Fragment<S>> {
        None
    }
}

/// Like nodes, marks (which are associated with nodes to signify
/// things like emphasis or being part of a link) are
/// [tagged](#model.Mark.type) with type objects, which are
/// instantiated once per `Schema`.
pub trait MarkType: Copy + Clone + Debug + PartialEq + Eq + PartialOrd + Ord {}
