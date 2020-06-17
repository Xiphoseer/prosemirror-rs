use super::{util::Span, StepKind};
use crate::model::{Fragment, Mark, MarkSet, Node, NodeType, Schema, Slice};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

fn map_fragment_parent<S: Schema, F>(fragment: &Fragment<S>, f: &F, parent: &S::Node) -> Fragment<S>
where
    F: Fn(S::Node, &S::Node, usize) -> S::Node,
{
    let mut mapped = vec![];
    for (i, child) in fragment.children().iter().enumerate() {
        let mut child = child.copy(|c| map_fragment_parent(c, f, &*child));

        if child.is_inline() {
            child = f(child, parent, i)
        }
        mapped.push(child)
    }
    Fragment::from(mapped)
}

fn map_fragment<S: Schema, F>(fragment: &Fragment<S>, f: &F) -> Fragment<S>
where
    F: Fn(S::Node) -> S::Node,
{
    let mut mapped = vec![];
    for child in fragment.children() {
        let mut child = child.copy(|c| map_fragment(c, f));

        if child.is_inline() {
            child = f(child)
        }
        mapped.push(child)
    }
    Fragment::from(mapped)
}

/// Adding a mark on some part of the document
#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug(bound = ""), PartialEq(bound = ""), Eq(bound = ""))]
#[serde(bound = "", rename_all = "camelCase")]
pub struct AddMarkStep<S: Schema> {
    /// The affected part of the document
    #[serde(flatten)]
    pub span: Span,
    /// The mark to add
    pub mark: S::Mark,
}

/// Removing a mark on some part of the document
#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Debug(bound = ""), PartialEq(bound = ""), Eq(bound = ""))]
#[serde(bound = "", rename_all = "camelCase")]
pub struct RemoveMarkStep<S: Schema> {
    /// The affected part of the document
    #[serde(flatten)]
    pub span: Span,
    /// The mark to remove
    pub mark: S::Mark,
}

impl<S: Schema> StepKind<S> for AddMarkStep<S> {
    fn apply(&self, doc: &S::Node) -> super::StepResult<S> {
        let old_slice = doc.slice(self.span.from..self.span.to, false)?;
        let rp_from = doc.resolve(self.span.from)?;
        let parent = rp_from.node(rp_from.shared_depth(self.span.to));

        let new_content = map_fragment_parent(
            &old_slice.content,
            &|node, parent, _i| {
                if parent.r#type().allows_mark_type(self.mark.r#type()) {
                    let new_marks = node.marks().map(Cow::Borrowed).unwrap_or_default();
                    node.mark(self.mark.add_to_set(new_marks).into_owned())
                } else {
                    node
                }
            },
            parent,
        );

        let slice = Slice::new(new_content, old_slice.open_start, old_slice.open_end);
        // TODO: Cow::Owned?
        let new_node = doc.replace(self.span.from..self.span.to, &slice)?;
        Ok(new_node)
    }
}

impl<S: Schema> StepKind<S> for RemoveMarkStep<S> {
    fn apply(&self, doc: &S::Node) -> super::StepResult<S> {
        let old_slice = doc.slice(self.span.from..self.span.to, false)?;

        let new_content = map_fragment(&old_slice.content, &|node| {
            let new_marks: Cow<MarkSet<S>> = node.marks().map(Cow::Borrowed).unwrap_or_default();
            node.mark(self.mark.remove_from_set(new_marks).into_owned())
        });

        let slice = Slice::new(new_content, old_slice.open_start, old_slice.open_end);
        let new_node = doc.replace(self.span.from..self.span.to, &slice)?;
        Ok(new_node)
    }
}
