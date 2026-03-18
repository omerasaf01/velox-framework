use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

/// Well-known HTTP status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NoContent = 204,
    MovedPermanently = 301,
    Found = 302,
    NotModified = 304,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    UnprocessableEntity = 422,
    TooManyRequests = 429,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
}

impl StatusCode {
    /// The numeric status code.
    pub fn code(self) -> u16 {
        self as u16
    }

    /// The standard reason phrase for this status code.
    pub fn reason(self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::Created => "Created",
            StatusCode::Accepted => "Accepted",
            StatusCode::NoContent => "No Content",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::Found => "Found",
            StatusCode::NotModified => "Not Modified",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::UnprocessableEntity => "Unprocessable Entity",
            StatusCode::TooManyRequests => "Too Many Requests",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::BadGateway => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
        }
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code(), self.reason())
    }
}

/// An HTTP response that will be sent back to the client.
#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    // ------------------------------------------------------------------
    // Constructors
    // ------------------------------------------------------------------

    /// Create a response with a custom status code and no body.
    pub fn new(status: StatusCode) -> Self {
        Response {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// `200 OK` with no body.
    pub fn ok() -> Self {
        Self::new(StatusCode::Ok)
    }

    /// `200 OK` with a plain-text body.
    pub fn text(body: impl Into<String>) -> Self {
        let body = body.into();
        let mut res = Self::new(StatusCode::Ok);
        res.headers
            .insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
        res.body = body.into_bytes();
        res
    }

    /// `200 OK` with a JSON body (the caller is responsible for valid JSON).
    pub fn json(body: impl Into<String>) -> Self {
        let body = body.into();
        let mut res = Self::new(StatusCode::Ok);
        res.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        res.body = body.into_bytes();
        res
    }

    /// `201 Created` with a JSON body.
    pub fn created(body: impl Into<String>) -> Self {
        let body = body.into();
        let mut res = Self::new(StatusCode::Created);
        res.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        res.body = body.into_bytes();
        res
    }

    /// `204 No Content`.
    pub fn no_content() -> Self {
        Self::new(StatusCode::NoContent)
    }

    /// `404 Not Found` with a plain-text message.
    pub fn not_found() -> Self {
        let mut res = Self::new(StatusCode::NotFound);
        res.headers
            .insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
        res.body = b"404 Not Found".to_vec();
        res
    }

    /// `405 Method Not Allowed`.
    pub fn method_not_allowed() -> Self {
        let mut res = Self::new(StatusCode::MethodNotAllowed);
        res.headers
            .insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
        res.body = b"405 Method Not Allowed".to_vec();
        res
    }

    /// `500 Internal Server Error` with an optional message.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        let msg = msg.into();
        let mut res = Self::new(StatusCode::InternalServerError);
        res.headers
            .insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
        res.body = msg.into_bytes();
        res
    }

    // ------------------------------------------------------------------
    // Builder methods (chainable)
    // ------------------------------------------------------------------

    /// Set a response header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Override the status code.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Replace the body with raw bytes.
    pub fn body_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.body = bytes;
        self
    }

    // ------------------------------------------------------------------
    // Serialisation
    // ------------------------------------------------------------------

    /// Serialise the response into raw bytes ready to be written to a TCP stream.
    pub fn into_bytes(mut self) -> Vec<u8> {
        // Always include Content-Length.
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());

        let mut head = String::new();
        write!(
            head,
            "HTTP/1.1 {}\r\n",
            self.status
        )
        .unwrap();

        for (name, value) in &self.headers {
            write!(head, "{}: {}\r\n", name, value).unwrap();
        }
        head.push_str("\r\n");

        let mut out = head.into_bytes();
        out.extend_from_slice(&self.body);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_response_bytes() {
        let bytes = Response::text("hello").into_bytes();
        let raw = String::from_utf8_lossy(&bytes);
        assert!(raw.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(raw.contains("Content-Length: 5"));
        assert!(raw.ends_with("hello"));
    }

    #[test]
    fn not_found_status() {
        let res = Response::not_found();
        assert_eq!(res.status.code(), 404);
    }

    #[test]
    fn status_code_display() {
        assert_eq!(StatusCode::Ok.to_string(), "200 OK");
        assert_eq!(StatusCode::NotFound.to_string(), "404 Not Found");
    }
}
