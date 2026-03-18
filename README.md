# Velox — a lightweight HTTP API framework in Rust

Velox is a minimal, educational HTTP/1.1 API framework written in pure Rust (zero
external dependencies). It is designed to be easy to read and understand — a great
starting point if you want to learn how web frameworks are built from scratch.

---

## Architecture

```
src/
├── lib.rs          # Public re-exports & crate documentation
├── error.rs        # VeloxError enum and Result<T> alias
├── request.rs      # HTTP request parser (method, path, headers, body, query params)
├── response.rs     # HTTP response builder (status codes, headers, body)
├── handler.rs      # Handler trait + FnHandler closure wrapper
├── middleware.rs   # Middleware trait + built-in Logger middleware
├── router.rs       # Route registration (:param paths) and dispatcher
└── server.rs       # TCP server (thread-per-connection)

examples/
└── basic.rs        # End-to-end example
```

---

## Quick Start

**1. Add Velox to your project**

```toml
[dependencies]
velox = { path = "." }
```

**2. Write your application**

```rust
use velox::handler::FnHandler;
use velox::middleware::Logger;
use velox::response::Response;
use velox::router::Router;
use velox::server::Server;

fn main() {
    let mut router = Router::new();

    // Middleware runs before every handler.
    router.middleware(Logger);

    // Static route.
    router.get("/", FnHandler::new(|_req| {
        Response::text("Hello, world!")
    }));

    // Dynamic route — :id is extracted as a path parameter.
    router.get("/users/:id", FnHandler::new(|req| {
        let id = req.param("id").unwrap_or("?");
        Response::json(format!(r#"{{"id": "{}"}}"#, id))
    }));

    // POST with body.
    router.post("/users", FnHandler::new(|req| {
        let body = req.body_str().unwrap_or("{}");
        Response::created(body)
    }));

    Server::new(router)
        .bind("127.0.0.1:7878")
        .expect("server failed");
}
```

**3. Run the built-in example**

```bash
cargo run --example basic
```

Then test it:

```bash
curl http://127.0.0.1:7878/
curl http://127.0.0.1:7878/users/42
curl -X POST http://127.0.0.1:7878/users -d '{"name":"Alice"}'
curl "http://127.0.0.1:7878/search?q=rust"
```

---

## Key Concepts

| Concept | Rust feature used | Where |
|---|---|---|
| HTTP parsing | `BufReader`, `Read`, `String` | `request.rs` |
| Composable responses | Builder pattern (chained `self`) | `response.rs` |
| Extensible handlers | Trait objects (`dyn Handler`) | `handler.rs` |
| Middleware chain | Recursive closures + traits | `middleware.rs` |
| Path parameters | Pattern matching on URL segments | `router.rs` |
| Concurrency | `Arc`, `thread::spawn` | `server.rs` |
| Error handling | `enum VeloxError`, `Result<T>` | `error.rs` |

---

## Running the Tests

```bash
cargo test
```

---

## License

MIT