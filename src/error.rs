#[derive(Debug)]
pub enum Error {
    Json(serde_json::Error),
    Unauthorized,
    BadRequest,
    NotFound,
    UnexpectedStatusCode(u16),
    MissingCookie,
    ClientError(String),
    NoSession,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Json(e) => write!(f, "JSON Error: {}", e),
            Error::Unauthorized => write!(f, "Unauthorized (401)"),
            Error::BadRequest => write!(f, "Bad Request (400)"),
            Error::NotFound => write!(f, "Not Found (404)"),
            Error::UnexpectedStatusCode(c) => write!(f, "Unexpected status code: {}", c),
            Error::MissingCookie => write!(f, "Set-Cookie header was missing from the response"),
            Error::ClientError(s) => write!(f, "Client Error: {}", s),
            Error::NoSession => write!(f, "No session available. Please login first."),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
