use std::collections::HashMap;
use std::sync::Arc;

use crate::handler::Handler;
use crate::middleware::Middleware;
use crate::request::{Method, Request};
use crate::response::Response;

/// A single registered route.
struct Route {
    method: Method,
    /// Path segments, where a segment starting with `:` is a parameter.
    /// e.g. `["users", ":id"]` matches `/users/42`.
    segments: Vec<String>,
    handler: Arc<dyn Handler>,
}

/// The router holds all registered routes and dispatches incoming requests to
/// the correct handler, extracting path parameters along the way.
pub struct Router {
    routes: Vec<Route>,
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl Router {
    /// Create an empty router.
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            middlewares: Vec::new(),
        }
    }

    // ------------------------------------------------------------------
    // Route registration helpers
    // ------------------------------------------------------------------

    /// Register a `GET` route.
    pub fn get<H: Handler + 'static>(&mut self, path: &str, handler: H) -> &mut Self {
        self.add_route(Method::Get, path, handler)
    }

    /// Register a `POST` route.
    pub fn post<H: Handler + 'static>(&mut self, path: &str, handler: H) -> &mut Self {
        self.add_route(Method::Post, path, handler)
    }

    /// Register a `PUT` route.
    pub fn put<H: Handler + 'static>(&mut self, path: &str, handler: H) -> &mut Self {
        self.add_route(Method::Put, path, handler)
    }

    /// Register a `PATCH` route.
    pub fn patch<H: Handler + 'static>(&mut self, path: &str, handler: H) -> &mut Self {
        self.add_route(Method::Patch, path, handler)
    }

    /// Register a `DELETE` route.
    pub fn delete<H: Handler + 'static>(&mut self, path: &str, handler: H) -> &mut Self {
        self.add_route(Method::Delete, path, handler)
    }

    fn add_route<H: Handler + 'static>(
        &mut self,
        method: Method,
        path: &str,
        handler: H,
    ) -> &mut Self {
        let segments = path_to_segments(path);
        self.routes.push(Route {
            method,
            segments,
            handler: Arc::new(handler),
        });
        self
    }

    // ------------------------------------------------------------------
    // Middleware registration
    // ------------------------------------------------------------------

    /// Add a middleware to the stack. Middleware is executed in the order it
    /// was added, wrapping every request/response.
    pub fn middleware<M: Middleware + 'static>(&mut self, m: M) -> &mut Self {
        self.middlewares.push(Arc::new(m));
        self
    }

    // ------------------------------------------------------------------
    // Dispatch
    // ------------------------------------------------------------------

    /// Dispatch a [`Request`] through the middleware stack and then to the
    /// matching route handler.  Returns the appropriate error response if no
    /// route matches.
    pub fn dispatch(&self, mut req: Request) -> Response {
        // Find a matching route.
        let match_result = self.find_route(&req);

        match match_result {
            None => Response::not_found(),
            Some(MatchResult::WrongMethod) => Response::method_not_allowed(),
            Some(MatchResult::Found { params, handler }) => {
                req.params = params;
                // Run the middleware chain.
                self.run_middlewares(&req, &*handler)
            }
        }
    }

    fn run_middlewares(&self, req: &Request, handler: &dyn Handler) -> Response {
        // Build the middleware chain from right to left so that the first
        // middleware added is the outermost (first to execute).
        fn build_chain(
            middlewares: &[Arc<dyn Middleware>],
            req: &Request,
            handler: &dyn Handler,
        ) -> Response {
            if middlewares.is_empty() {
                return handler.handle(req);
            }
            let (head, tail) = middlewares.split_first().unwrap();
            let tail_ref = tail;
            head.call(req, &|r| build_chain(tail_ref, r, handler))
        }

        build_chain(&self.middlewares, req, handler)
    }

    fn find_route(&self, req: &Request) -> Option<MatchResult> {
        let request_segments = path_to_segments(&req.path);
        let mut method_matched = false;

        for route in &self.routes {
            if let Some(params) = match_segments(&route.segments, &request_segments) {
                if route.method == req.method {
                    return Some(MatchResult::Found {
                        params,
                        handler: Arc::clone(&route.handler),
                    });
                }
                method_matched = true;
            }
        }

        if method_matched {
            Some(MatchResult::WrongMethod)
        } else {
            None
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

enum MatchResult {
    Found {
        params: HashMap<String, String>,
        handler: Arc<dyn Handler>,
    },
    WrongMethod,
}

/// Split a URL path into segments, ignoring leading/trailing slashes.
fn path_to_segments(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Try to match `route_segs` against `request_segs`.
/// Returns `Some(params)` on success, `None` on mismatch.
fn match_segments(
    route_segs: &[String],
    request_segs: &[String],
) -> Option<HashMap<String, String>> {
    if route_segs.len() != request_segs.len() {
        return None;
    }

    let mut params = HashMap::new();
    for (r, q) in route_segs.iter().zip(request_segs.iter()) {
        if let Some(param_name) = r.strip_prefix(':') {
            params.insert(param_name.to_string(), q.clone());
        } else if r != q {
            return None;
        }
    }
    Some(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::FnHandler;

    fn make_router() -> Router {
        let mut router = Router::new();
        router.get("/hello", FnHandler::new(|_req| Response::text("hello")));
        router.get(
            "/users/:id",
            FnHandler::new(|req| {
                let id = req.param("id").unwrap_or("?");
                Response::text(format!("user {}", id))
            }),
        );
        router.post("/users", FnHandler::new(|_req| Response::created("{}")));
        router
    }

    fn fake_request(method: Method, path: &str) -> Request {
        Request {
            method,
            path: path.to_string(),
            query_string: String::new(),
            query_params: HashMap::new(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            params: HashMap::new(),
            body: Vec::new(),
        }
    }

    #[test]
    fn get_hello() {
        let router = make_router();
        let req = fake_request(Method::Get, "/hello");
        let res = router.dispatch(req);
        assert_eq!(res.status.code(), 200);
    }

    #[test]
    fn route_param_extraction() {
        let router = make_router();
        let req = fake_request(Method::Get, "/users/42");
        let res = router.dispatch(req);
        assert_eq!(res.status.code(), 200);
        assert_eq!(String::from_utf8_lossy(&res.body), "user 42");
    }

    #[test]
    fn not_found() {
        let router = make_router();
        let req = fake_request(Method::Get, "/missing");
        let res = router.dispatch(req);
        assert_eq!(res.status.code(), 404);
    }

    #[test]
    fn method_not_allowed() {
        let router = make_router();
        // /hello is registered only for GET.
        let req = fake_request(Method::Post, "/hello");
        let res = router.dispatch(req);
        assert_eq!(res.status.code(), 405);
    }

    #[test]
    fn post_route() {
        let router = make_router();
        let req = fake_request(Method::Post, "/users");
        let res = router.dispatch(req);
        assert_eq!(res.status.code(), 201);
    }
}
