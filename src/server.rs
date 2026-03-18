use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use crate::error::{Result, VeloxError};
use crate::request::Request;
use crate::router::Router;

// ---------------------------------------------------------------------------
// Thread pool
// ---------------------------------------------------------------------------

type Job = Box<dyn FnOnce() + Send + 'static>;

/// A fixed-size pool of worker threads.
///
/// Jobs submitted via [`ThreadPool::execute`] are queued and picked up by the
/// next available worker, bounding the total number of concurrent threads.
struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    /// Create a pool with `size` worker threads.
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero.
    fn new(size: usize) -> Self {
        assert!(size > 0, "thread pool size must be greater than 0");

        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|_| {
                let rx = Arc::clone(&receiver);
                thread::spawn(move || loop {
                    // Block until a job is available or the channel closes.
                    let job = match rx.lock().unwrap().recv() {
                        Ok(job) => job,
                        Err(_) => break, // sender dropped → shut down
                    };
                    job();
                })
            })
            .collect();

        ThreadPool { workers, sender }
    }

    /// Submit a job to be run by the next available worker thread.
    fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        // If all workers are busy the job is queued in the channel.
        if self.sender.send(Box::new(f)).is_err() {
            eprintln!("[velox] thread pool: failed to queue job (pool shut down)");
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Dropping `sender` closes the channel; workers will exit their loops.
        // We move the handles out so we can join them.
        let handles: Vec<_> = self.workers.drain(..).collect();
        drop(self.sender.clone()); // signal workers to stop
        for handle in handles {
            let _ = handle.join();
        }
    }
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

/// The Velox HTTP server.
///
/// Binds a TCP socket, accepts connections, and dispatches each request
/// through the [`Router`].  Connections are handled by a thread pool so that
/// the number of concurrent OS threads stays bounded.
///
/// # Example
///
/// ```rust,no_run
/// use velox::handler::FnHandler;
/// use velox::response::Response;
/// use velox::router::Router;
/// use velox::server::Server;
///
/// let mut router = Router::new();
/// router.get("/", FnHandler::new(|_req| Response::text("Hello, world!")));
///
/// Server::new(router)
///     .bind("127.0.0.1:7878")
///     .expect("failed to start server");
/// ```
pub struct Server {
    router: Arc<Router>,
    /// Number of worker threads in the connection pool.
    workers: usize,
}

impl Server {
    /// Create a new server with the given router.
    ///
    /// Defaults to `num_cpus * 2` worker threads (minimum 4).
    pub fn new(router: Router) -> Self {
        let workers = (thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2)
            * 2)
        .max(4);

        Server {
            router: Arc::new(router),
            workers,
        }
    }

    /// Override the number of worker threads (must be ≥ 1).
    pub fn workers(mut self, n: usize) -> Self {
        self.workers = n.max(1);
        self
    }

    /// Bind to `addr` and start serving requests.
    ///
    /// This call **blocks** the current thread indefinitely (until the process
    /// is killed or the listener errors out).
    pub fn bind(self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).map_err(VeloxError::Io)?;
        let pool = ThreadPool::new(self.workers);
        println!(
            "[velox] Listening on http://{} ({} workers)",
            addr, self.workers
        );

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let router = Arc::clone(&self.router);
                    pool.execute(move || {
                        if let Err(err) = handle_connection(stream, router) {
                            eprintln!("[velox] connection error: {}", err);
                        }
                    });
                }
                Err(err) => {
                    eprintln!("[velox] accept error: {}", err);
                }
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn handle_connection(mut stream: TcpStream, router: Arc<Router>) -> Result<()> {
    let request = match Request::from_stream(&stream) {
        Ok(req) => req,
        Err(err) => {
            eprintln!("[velox] failed to parse request: {}", err);
            // Try to send a 400 Bad Request back to the client.
            let response = crate::response::Response::new(crate::response::StatusCode::BadRequest);
            if let Err(write_err) = stream.write_all(&response.into_bytes()) {
                eprintln!("[velox] failed to write 400 response: {}", write_err);
            }
            return Ok(());
        }
    };

    let response = router.dispatch(request);
    stream
        .write_all(&response.into_bytes())
        .map_err(VeloxError::Io)?;

    Ok(())
}
