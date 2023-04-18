#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    RequestError(String),
    ResponseError(String),
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
