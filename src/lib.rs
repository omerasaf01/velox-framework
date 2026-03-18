//! # Velox — a lightweight, educational HTTP API framework in Rust
//!
//! Velox provides the building blocks you need to handle HTTP/1.1 requests
//! and return responses:
//!
//! | Module | What it gives you |
//! |---|---|
//! | [`request`] | Parse an HTTP request from a TCP stream |
//! | [`response`] | Build HTTP responses with status codes and headers |
//! | [`handler`] | A trait + closure wrapper for request handlers |
//! | [`middleware`] | A trait for composable middleware |
//! | [`router`] | Register routes and dispatch requests |
//! | [`server`] | A TCP server that ties everything together |
//! | [`error`] | Framework error types |
//!
//! ## Quick-start
//!
//! ```rust,no_run
//! use velox::handler::FnHandler;
//! use velox::middleware::Logger;
//! use velox::response::Response;
//! use velox::router::Router;
//! use velox::server::Server;
//!
//! fn main() {
//!     let mut router = Router::new();
//!
//!     router.middleware(Logger);
//!
//!     router.get("/", FnHandler::new(|_req| {
//!         Response::text("Hello, world!")
//!     }));
//!
//!     router.get("/users/:id", FnHandler::new(|req| {
//!         let id = req.param("id").unwrap_or("unknown");
//!         Response::json(format!(r#"{{"id": "{}"}}"#, id))
//!     }));
//!
//!     Server::new(router)
//!         .bind("127.0.0.1:7878")
//!         .expect("server failed");
//! }
//! ```

pub mod error;
pub mod handler;
pub mod middleware;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
