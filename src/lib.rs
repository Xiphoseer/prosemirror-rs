#![warn(missing_docs)]
//! # The ProseMirror API
//!
//! This crate is a re-implementation of the [ProseMirror](https://prosemirror.net) API in Rust.
//! It can be used to create a collaborative editing authority that is able to apply steps to
//! a document.

pub(crate) mod de;
pub mod markdown;
pub mod model;
pub mod transform;
