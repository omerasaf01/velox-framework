use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

use crate::error::{Result, VeloxError};

/// Supported HTTP methods.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    /// Any non-standard or unrecognised method string.
    Other(String),
}

impl Method {
    /// Parse a raw method string (case-insensitive).
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            "DELETE" => Method::Delete,
            "HEAD" => Method::Head,
            "OPTIONS" => Method::Options,
            other => Method::Other(other.to_string()),
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
            Method::Other(v) => v.as_str(),
        };
        write!(f, "{}", s)
    }
}

/// A parsed HTTP/1.1 request.
#[derive(Debug)]
pub struct Request {
    /// HTTP method (GET, POST, …).
    pub method: Method,
    /// The request path, e.g. `/users/42`.
    pub path: String,
    /// Raw query string, e.g. `page=1&limit=10`.
    pub query_string: String,
    /// Parsed query parameters.
    pub query_params: HashMap<String, String>,
    /// HTTP version string, e.g. `HTTP/1.1`.
    pub version: String,
    /// Request headers (lower-cased names).
    pub headers: HashMap<String, String>,
    /// Route parameters extracted by the router (e.g. `{id}` → `"42"`).
    pub params: HashMap<String, String>,
    /// Raw request body bytes.
    pub body: Vec<u8>,
}

impl Request {
    /// Read and parse an HTTP/1.1 request from a TCP stream.
    pub fn from_stream(stream: &TcpStream) -> Result<Self> {
        let mut reader = BufReader::new(stream);

        // --- Request line ---
        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .map_err(VeloxError::Io)?;
        let request_line = request_line.trim();

        let mut parts = request_line.splitn(3, ' ');
        let method_str = parts
            .next()
            .ok_or_else(|| VeloxError::ParseError("missing method".to_string()))?;
        let raw_path = parts
            .next()
            .ok_or_else(|| VeloxError::ParseError("missing path".to_string()))?;
        let version = parts
            .next()
            .ok_or_else(|| VeloxError::ParseError("missing HTTP version".to_string()))?
            .to_string();

        let method = Method::from_str(method_str);

        // Split path and query string.
        let (path, query_string) = if let Some(idx) = raw_path.find('?') {
            (
                raw_path[..idx].to_string(),
                raw_path[idx + 1..].to_string(),
            )
        } else {
            (raw_path.to_string(), String::new())
        };

        let query_params = parse_query(&query_string);

        // --- Headers ---
        let mut headers: HashMap<String, String> = HashMap::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).map_err(VeloxError::Io)?;
            let line = line.trim();
            if line.is_empty() {
                break; // blank line separates headers from body
            }
            if let Some(colon) = line.find(':') {
                let name = line[..colon].trim().to_lowercase();
                let value = line[colon + 1..].trim().to_string();
                headers.insert(name, value);
            }
        }

        // --- Body ---
        let body = if let Some(len_str) = headers.get("content-length") {
            let len: usize = len_str
                .parse()
                .map_err(|_| VeloxError::ParseError("invalid content-length".to_string()))?;
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf).map_err(VeloxError::Io)?;
            buf
        } else {
            Vec::new()
        };

        Ok(Request {
            method,
            path,
            query_string,
            query_params,
            version,
            headers,
            params: HashMap::new(),
            body,
        })
    }

    /// Return the body as a UTF-8 string slice (if valid).
    pub fn body_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }

    /// Look up a header by name (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|v| v.as_str())
    }

    /// Look up a route parameter by name.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(|v| v.as_str())
    }

    /// Look up a query parameter by name.
    pub fn query(&self, name: &str) -> Option<&str> {
        self.query_params.get(name).map(|v| v.as_str())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a URL-encoded query string into a key-value map.
fn parse_query(qs: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if qs.is_empty() {
        return map;
    }
    for pair in qs.split('&') {
        let mut kv = pair.splitn(2, '=');
        if let Some(key) = kv.next() {
            let value = kv.next().unwrap_or("").to_string();
            map.insert(url_decode(key), url_decode(&value));
        }
    }
    map
}

/// Minimal percent-decoding for query parameters (replaces `+` with space and
/// decodes `%XX` sequences).
fn url_decode(s: &str) -> String {
    let s = s.replace('+', " ");
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_from_str() {
        assert_eq!(Method::from_str("GET"), Method::Get);
        assert_eq!(Method::from_str("post"), Method::Post);
        assert_eq!(Method::from_str("PATCH"), Method::Patch);
        assert_eq!(
            Method::from_str("CUSTOM"),
            Method::Other("CUSTOM".to_string())
        );
    }

    #[test]
    fn query_parsing() {
        let params = parse_query("page=1&limit=20&q=hello+world");
        assert_eq!(params.get("page").map(|s| s.as_str()), Some("1"));
        assert_eq!(params.get("limit").map(|s| s.as_str()), Some("20"));
        assert_eq!(params.get("q").map(|s| s.as_str()), Some("hello world"));
    }

    #[test]
    fn url_decode_percent() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("caf%C3%A9"), "café");
    }
}
