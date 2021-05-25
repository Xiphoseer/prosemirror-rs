use super::{Span, StepError, StepKind, StepResult};
use prosemirror_model::{Node, ResolveErr, Schema, Slice};
use derivative::Derivative;
use serde::{Deserialize, Serialize};

/// Replace some part of the document
#[derive(Debug, Derivative, Deserialize, Serialize)]
#[derivative(PartialEq(bound = ""), Eq(bound = ""))]
#[serde(bound = "", rename_all = "camelCase")]
pub struct ReplaceStep<S: Schema> {
    /// The affected span
    #[serde(flatten)]
    pub span: Span,
    /// The slice to replace the current content with
    #[serde(default)]
    pub slice: Slice<S>,
    /// Whether this is a structural change
    #[serde(default)]
    pub structure: bool,
}

impl<S: Schema> StepKind<S> for ReplaceStep<S> {
    fn apply(&self, doc: &S::Node) -> StepResult<S> {
        let from = self.span.from;
        let to = self.span.to;
        if self.structure && content_between::<S>(doc, from, to)? {
            Err(StepError::WouldOverwrite)
        } else {
            let node = doc.replace(from..to, &self.slice)?;
            Ok(node)
        }
    }
}

/// Replace the document structure while keeping some content
#[derive(Debug, Derivative, Deserialize, Serialize)]
#[derivative(PartialEq(bound = ""), Eq(bound = ""))]
#[serde(bound = "", rename_all = "camelCase")]
pub struct ReplaceAroundStep<S: Schema> {
    /// The affected part of the document
    #[serde(flatten)]
    pub span: Span,
    /// Start of the gap
    pub gap_from: usize,
    /// End of the gap
    pub gap_to: usize,
    /// The inner slice
    #[serde(default)]
    pub slice: Slice<S>,
    /// ???
    pub insert: usize,
    /// Whether this is a structural change
    #[serde(default)]
    pub structure: bool,
}

impl<S: Schema> StepKind<S> for ReplaceAroundStep<S> {
    fn apply(&self, doc: &S::Node) -> StepResult<S> {
        if self.structure
            && (content_between::<S>(doc, self.span.from, self.gap_from)?
                || content_between::<S>(doc, self.gap_to, self.span.to)?)
        {
            return Err(StepError::GapWouldOverwrite);
        }

        let gap = doc.slice(self.gap_from..self.gap_to, false)?;
        if gap.open_start != 0 || gap.open_end != 0 {
            return Err(StepError::GapNotFlat);
        }

        let inserted = self.slice.insert_at(self.insert, gap.content)?;
        let inserted = inserted.ok_or(StepError::GapNotFit)?;

        let result = doc.replace(self.span.from..self.span.to, &inserted)?;
        Ok(result)
    }
}

fn content_between<S: Schema>(doc: &S::Node, from: usize, to: usize) -> Result<bool, ResolveErr> {
    let rp_from = doc.resolve(from)?;
    let mut dist = to - from;
    let mut depth = rp_from.depth();
    while dist > 0 && depth > 0 && rp_from.index_after(depth) == rp_from.node(depth).child_count() {
        depth -= 1;
        dist -= 1;
    }
    if dist > 0 {
        let mut next = rp_from.node(depth).maybe_child(rp_from.index_after(depth));
        while dist > 0 {
            match next {
                Some(c) => {
                    if c.is_leaf() {
                        return Ok(true);
                    } else {
                        next = c.first_child();
                        dist -= 1;
                    }
                }
                None => {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
