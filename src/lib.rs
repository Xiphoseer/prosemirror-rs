#![warn(missing_docs)]
//! # The ProseMirror API
//!
//! This crate is a re-implementation of the [ProseMirror](https://prosemirror.net) API in Rust.
//! It can be used to create a collaborative editing authority that is able to apply steps to
//! a document.

#[cfg(test)]
mod tests;

pub use prosemirror_model as model;
pub use prosemirror_transform as transform;
