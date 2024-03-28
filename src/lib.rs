//! atticus: A simple implementation of an actor in Tokio.
//!
//! Actors provide a way to invoke messages or requests among asynchronous tasks.  This avoids the
//! need to use `Arc<Mutex<T>>` instances of an object to be passed around so shared state can be
//! made. It makes use of channels to exchange data.
//!
//! Actors aim to clarify ownership data structures.
//!
//! Create an actor by implementing the [Actor] trait.
//!
//! ```rust
//! use atticus::Actor;
//! use async_trait::async_trait;
//!
//! struct IntToString;
//!
//! #[async_trait]
//! impl Actor for IntToString {
//!    type Request = i32;
//!    type Response = String;
//!    async fn handle(&mut self, request: Self::Request) -> Option<Self::Response> {
//!        Some(request.to_string())
//!    }
//! }
//!
//! #[tokio::main(flavor="current_thread")]
//! async fn main() {
//!    // Spawn using [run_actor]
//!    let handle = atticus::run_actor(IntToString{}, 1);
//!
//!    // Send a request to convert 5 to String.
//!    let response = handle.requestor.request(5).await;
//!
//!    assert!(response.is_ok());
//!    assert_eq!(response.unwrap(), Some(String::from("5")));
//! }
//!
//! ```

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// Defines the interface on how to glue in your custom [Actor]
pub mod actor;

/// Defines the error types that this crate provides.
pub mod error;

pub use actor::{run_actor, Actor, Requestor};
pub use error::Error;

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::actor::*;
    use crate::Error;

    #[derive(Debug)]
    struct NormalType;

    #[async_trait]
    impl Actor for NormalType {
        type Request = usize;
        type Response = bool;

        async fn handle(&mut self, _message: Self::Request) -> Option<Self::Response> {
            unreachable!()
        }
    }

    const fn is_send_sync<T: Sized + Send + Sync + Unpin>() {}
    const fn impls_or_derives_debug<Debug>() {}

    #[test]
    const fn test_public_types() {
        is_send_sync::<Requestor<NormalType, NormalType>>();
        is_send_sync::<Handle<NormalType>>();
        is_send_sync::<Error>();
    }

    #[test]
    const fn test_impls_debug() {
        impls_or_derives_debug::<Requestor<NormalType, NormalType>>();
        impls_or_derives_debug::<Handle<NormalType>>();
        impls_or_derives_debug::<Error>();
    }
}
