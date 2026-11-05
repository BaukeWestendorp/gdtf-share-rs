//! Errors for GDTF Share interaction.

/// Errors that can occur during GDTF Share API interaction.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// JSON serialization or deserialization failure.
    Json(serde_json::Error),
    /// Request was unauthorized (HTTP 401).
    Unauthorized,
    /// Client issued a bad request (HTTP 400).
    BadRequest,
    /// The requested resource was not found (HTTP 404).
    NotFound,
    /// The server responded with an unexpected status code.
    UnexpectedStatusCode(u16),
    /// The `Set-Cookie` header was missing from the response.
    MissingCookie,
    /// The rating value is invalid.
    InvalidRating,
    /// Transport or underlying client error.
    ClientError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(e) => write!(f, "JSON Error: {e}"),
            Self::Unauthorized => write!(f, "Unauthorized (401)"),
            Self::BadRequest => write!(f, "Bad Request (400)"),
            Self::NotFound => write!(f, "Not Found (404)"),
            Self::UnexpectedStatusCode(c) => write!(f, "Unexpected status code: {c}"),
            Self::MissingCookie => write!(f, "Set-Cookie header was missing from the response"),
            Self::InvalidRating => write!(f, "Invalid rating value"),
            Self::ClientError(s) => write!(f, "Client Error: {s}"),
        }
    }
}

impl std::error::Error for Error {}

/// Type result alias wrapping [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
