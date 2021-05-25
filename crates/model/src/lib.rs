//! # The document model
//!
//! This module is derived from the `prosemirror-markdown` schema and the
//! the general JSON serialization of nodes.
mod content;
mod fragment;
mod marks;
mod node;
mod replace;
mod resolved_pos;
mod schema;
pub(crate) mod util;

pub use content::{ContentMatch, ContentMatchError};
pub use fragment::Fragment;
pub use marks::{Mark, MarkSet};
pub use node::{Node, NodeType, SliceError, Text};
pub use replace::{InsertError, ReplaceError, Slice};
pub use resolved_pos::{ResolveErr, ResolvedNode, ResolvedPos};
pub use schema::{AttrNode, Block, Leaf, MarkType, Schema, TextNode, NodeImpl};

pub(crate) use replace::replace;
pub(crate) use resolved_pos::Index;