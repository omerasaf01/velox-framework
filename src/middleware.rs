use crate::request::Request;
use crate::response::Response;

/// Middleware sits in between the server and the final route handler.
///
/// Every middleware receives the [`Request`] and a `next` callable that
/// forwards execution to the remaining middleware chain (and eventually the
/// handler).  The middleware may inspect or modify the request, short-circuit
/// with its own response, or post-process the response returned by `next`.
///
/// # Example – simple logging middleware
///
/// ```rust
/// use velox::middleware::Middleware;
/// use velox::request::Request;
/// use velox::response::Response;
///
/// pub struct Logger;
///
/// impl Middleware for Logger {
///     fn call(&self, req: &Request, next: &dyn Fn(&Request) -> Response) -> Response {
///         println!("--> {} {}", req.method, req.path);
///         let res = next(req);
///         println!("<-- {}", res.status);
///         res
///     }
/// }
/// ```
pub trait Middleware: Send + Sync {
    fn call(&self, req: &Request, next: &dyn Fn(&Request) -> Response) -> Response;
}

// ---------------------------------------------------------------------------
// Built-in middleware
// ---------------------------------------------------------------------------

/// Logs every request and its response status to stdout.
pub struct Logger;

impl Middleware for Logger {
    fn call(&self, req: &Request, next: &dyn Fn(&Request) -> Response) -> Response {
        println!("[velox] --> {} {}", req.method, req.path);
        let res = next(req);
        println!("[velox] <-- {}", res.status);
        res
    }
}
