use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
/// The types of errors that this crate could indicate
pub enum Error {
    #[error("Problem in sending a Request to the Actor")]
    /// A problem was encounted during `send()`ing of the request
    RequestError,

    #[error("Problem in receiving a Response from the Actor")]
    /// An error was encounted while waiting for the response from the `Actor` instance.
    ResponseError,

    #[error("Problem in sending an event to the Actor")]
    /// A problem was encounted in sending an event
    EventError,
}
