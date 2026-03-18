use std::fmt;

/// All errors that can occur within the Velox framework.
#[derive(Debug)]
pub enum VeloxError {
    /// The incoming HTTP request could not be parsed.
    ParseError(String),
    /// A required header was missing from the request.
    MissingHeader(String),
    /// An I/O error occurred (e.g. reading from the TCP stream).
    Io(std::io::Error),
    /// The requested route was not found.
    NotFound,
    /// The HTTP method used is not allowed for this route.
    MethodNotAllowed,
    /// An arbitrary internal error produced by a handler or middleware.
    Internal(String),
}

impl fmt::Display for VeloxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VeloxError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            VeloxError::MissingHeader(name) => write!(f, "Missing header: {}", name),
            VeloxError::Io(err) => write!(f, "IO error: {}", err),
            VeloxError::NotFound => write!(f, "Not found"),
            VeloxError::MethodNotAllowed => write!(f, "Method not allowed"),
            VeloxError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for VeloxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VeloxError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for VeloxError {
    fn from(err: std::io::Error) -> Self {
        VeloxError::Io(err)
    }
}

/// A convenient type alias used throughout the framework.
pub type Result<T> = std::result::Result<T, VeloxError>;
