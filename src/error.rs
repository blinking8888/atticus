#[derive(Debug, Clone, PartialEq, Eq)]
/// The types of errors that this crate could indicate
pub enum Error {
    /// A problem was encounted during `send()`ing of the request
    RequestError(String),
    /// An error was encounted while waiting for the response from the `Actor` instance.
    ResponseError(String),
    /// A problem was encounted in sending an event
    EventError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        let display = match self {
            RequestError(ref s) | ResponseError(ref s) | EventError(ref s) => s,
        };
        write!(f, "{}", display)
    }
}

impl std::error::Error for Error {}
