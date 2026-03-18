use crate::request::Request;
use crate::response::Response;

/// A handler is any type that can process an HTTP [`Request`] and produce a [`Response`].
///
/// Implement this trait for your own handler structs, or use the provided
/// [`FnHandler`] wrapper to use plain functions / closures.
///
/// # Example
///
/// ```rust
/// use velox::handler::Handler;
/// use velox::request::Request;
/// use velox::response::Response;
///
/// struct HelloHandler;
///
/// impl Handler for HelloHandler {
///     fn handle(&self, req: &Request) -> Response {
///         Response::text(format!("Hello from {}", req.path))
///     }
/// }
/// ```
pub trait Handler: Send + Sync {
    fn handle(&self, req: &Request) -> Response;
}

/// A [`Handler`] implementation that wraps a plain function or closure.
///
/// This lets you register ordinary `fn` items and closures as route handlers
/// without defining a new struct.
pub struct FnHandler<F>
where
    F: Fn(&Request) -> Response + Send + Sync,
{
    f: F,
}

impl<F> FnHandler<F>
where
    F: Fn(&Request) -> Response + Send + Sync,
{
    /// Wrap a function or closure.
    pub fn new(f: F) -> Self {
        FnHandler { f }
    }
}

impl<F> Handler for FnHandler<F>
where
    F: Fn(&Request) -> Response + Send + Sync,
{
    fn handle(&self, req: &Request) -> Response {
        (self.f)(req)
    }
}
