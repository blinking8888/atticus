//! simple-actor: A simple implementation of an actor in Tokio.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// [actor] defines the interface on how to glue in your custom [Actor]
pub mod actor;

/// [error] defines the error types that this crate provides.
pub mod error;

pub use actor::{run_actor, Actor};
