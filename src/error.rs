use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum WebArenaIndigoApiError {
    /// Failed to send the request (network / TLS / timeout).
    Connection(String),
    /// The API returned a non-2xx status. Holds the status code and the raw response body.
    Status(u16, String),
    /// The response body could not be parsed. Holds the raw response body.
    JsonParse(String),
}

impl Display for WebArenaIndigoApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WebArenaIndigoApiError::Connection(e) => write!(f, "connection: {}", e),
            WebArenaIndigoApiError::Status(status, body) => {
                write!(f, "status: {}, message: {}", status, body)
            }
            WebArenaIndigoApiError::JsonParse(body) => write!(f, "json parse: {}", body),
        }
    }
}

impl Error for WebArenaIndigoApiError {}
