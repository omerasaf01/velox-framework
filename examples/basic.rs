//! A minimal end-to-end example of the Velox framework.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example basic
//! ```
//!
//! Then try:
//!
//! ```bash
//! curl http://127.0.0.1:7878/
//! curl http://127.0.0.1:7878/hello
//! curl http://127.0.0.1:7878/users/42
//! curl -X POST http://127.0.0.1:7878/users \
//!      -H 'Content-Type: application/json' \
//!      -d '{"name":"Alice"}'
//! ```

use velox::handler::FnHandler;
use velox::middleware::Logger;
use velox::response::Response;
use velox::router::Router;
use velox::server::Server;

fn main() {
    let mut router = Router::new();

    // Global middleware – logs every request.
    router.middleware(Logger);

    // GET /
    router.get(
        "/",
        FnHandler::new(|_req| Response::text("Welcome to Velox!")),
    );

    // GET /hello
    router.get(
        "/hello",
        FnHandler::new(|_req| Response::text("Hello, world!")),
    );

    // GET /users/:id  — demonstrates path parameter extraction.
    router.get(
        "/users/:id",
        FnHandler::new(|req| {
            let id = req.param("id").unwrap_or("unknown");
            Response::json(format!(r#"{{"id": "{}"}}"#, id))
        }),
    );

    // POST /users  — demonstrates reading the request body.
    router.post(
        "/users",
        FnHandler::new(|req| {
            let body = req.body_str().unwrap_or("{}");
            println!("[handler] received body: {}", body);
            Response::created(format!(r#"{{"created": true, "data": {}}}"#, body))
        }),
    );

    // GET /search?q=…  — demonstrates query parameters.
    router.get(
        "/search",
        FnHandler::new(|req| {
            let q = req.query("q").unwrap_or("");
            Response::json(format!(r#"{{"query": "{}"}}"#, q))
        }),
    );

    println!("Starting Velox on http://127.0.0.1:7878");
    Server::new(router)
        .bind("127.0.0.1:7878")
        .expect("failed to start server");
}
