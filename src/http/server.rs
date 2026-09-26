//! Connection acceptance and dispatch to a bounded worker pool.

use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use super::request::{Limits, Method, Request, RequestError, read_request};
use super::response::{Response, Status};

/// A request handler.
///
/// The handler receives the request plus the peer address and returns the
/// response to send. It must not block on shared state that another handler
/// could hold for long, because a bounded number of requests are served
/// concurrently.
pub trait Handler: Send + Sync + 'static {
    /// Produces the response for one request.
    fn handle(&self, request: &Request, peer: SocketAddr) -> Response;
}

impl<F> Handler for F
where
    F: Fn(&Request, SocketAddr) -> Response + Send + Sync + 'static,
{
    fn handle(&self, request: &Request, peer: SocketAddr) -> Response {
        self(request, peer)
    }
}

/// Tuning knobs for the server.
#[derive(Debug, Clone, Copy)]
pub struct ServerConfig {
    /// Number of worker threads.
    pub workers: usize,
    /// Maximum number of connections waiting for a worker.
    pub backlog: usize,
    /// Read and write timeout applied to every connection.
    pub timeout: Duration,
    /// Request size limits.
    pub limits: Limits,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            workers: 4,
            backlog: 128,
            timeout: Duration::from_secs(15),
            limits: Limits::default(),
        }
    }
}

/// A running HTTP server.
pub struct Server {
    listener: TcpListener,
    config: ServerConfig,
}

impl Server {
    /// Binds a server to `address`.
    ///
    /// The address accepts anything that resolves to a socket address, such
    /// as `"127.0.0.1:8080"` or `"0.0.0.0:80"`.
    pub fn bind(address: &str, config: ServerConfig) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        Ok(Self { listener, config })
    }

    /// Returns the address the server is bound to.
    ///
    /// This is useful when the configured port is zero and the operating
    /// system assigned one.
    pub fn local_address(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Runs the accept loop until the listener fails.
    ///
    /// Every accepted connection is handed to a worker thread from a fixed
    /// size pool. When all workers are busy a connection waits in the queue,
    /// so memory use stays bounded by the pool size and the queue capacity.
    pub fn run<H: Handler>(self, handler: Arc<H>) -> io::Result<()> {
        let (sender, receiver) = mpsc::sync_channel::<TcpStream>(self.config.backlog);
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = self.config.workers.max(1);

        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let receiver = Arc::clone(&receiver);
            let handler = Arc::clone(&handler);
            let config = self.config;
            handles.push(
                thread::Builder::new()
                    .name("apsis-book-worker".to_string())
                    .spawn(move || worker_loop(&receiver, &*handler, config))?,
            );
        }

        for connection in self.listener.incoming() {
            match connection {
                Ok(stream) => {
                    // A disconnected worker pool means the process is
                    // shutting down; dropping the stream closes it.
                    if sender.send(stream).is_err() {
                        break;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }

        drop(sender);
        for handle in handles {
            let _ = handle.join();
        }
        Ok(())
    }
}

/// Serves requests from the queue until every sender is dropped.
fn worker_loop<H: Handler>(
    receiver: &Mutex<mpsc::Receiver<TcpStream>>,
    handler: &H,
    config: ServerConfig,
) {
    loop {
        // The lock is released before the request is served, so workers do
        // not serialise on the queue.
        let stream = {
            let guard = match receiver.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            guard.recv()
        };
        let Ok(stream) = stream else {
            return;
        };
        serve_connection(stream, handler, config);
    }
}

/// Reads one request from a connection and writes the response.
fn serve_connection<H: Handler>(mut stream: TcpStream, handler: &H, config: ServerConfig) {
    let peer = stream
        .peer_addr()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));
    let _ = stream.set_read_timeout(Some(config.timeout));
    let _ = stream.set_write_timeout(Some(config.timeout));
    let _ = stream.set_nodelay(true);

    let response = match read_request(&mut stream, config.limits) {
        Ok(request) => {
            let suppress_body = request.method == Method::Head;
            let response = dispatch(handler, &request, peer);
            let _ = response.write_to(&mut stream, suppress_body);
            return;
        }
        Err(error) => error_response(error),
    };
    let _ = response.write_to(&mut stream, false);
}

/// Turns a request parsing failure into a response.
///
/// The body describes the class of failure without repeating the input, so
/// that a malformed request cannot be used to reflect arbitrary text.
fn error_response(error: RequestError) -> Response {
    match error {
        RequestError::TooLarge(detail) => Response::error(Status::ContentTooLarge, detail),
        RequestError::Malformed(detail) => Response::error(Status::BadRequest, detail),
        RequestError::Io(_) => {
            Response::error(Status::BadRequest, "the connection could not be read")
        }
    }
}

/// Calls the handler, converting a panic into a server error response.
///
/// A handler is the only place where an unexpected condition can surface, so
/// it is the only place that needs this guard. Catching the panic keeps one
/// bad request from tearing down a worker thread.
fn dispatch<H: Handler>(handler: &H, request: &Request, peer: SocketAddr) -> Response {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        handler.handle(request, peer)
    }))
    .unwrap_or_else(|_| Response::error(Status::InternalServerError, "internal server error"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn serves_a_request_over_a_real_socket() {
        let server = Server::bind(
            "127.0.0.1:0",
            ServerConfig {
                workers: 2,
                ..ServerConfig::default()
            },
        )
        .unwrap();
        let address = server.local_address().unwrap();

        let handler =
            Arc::new(|_request: &Request, _peer: SocketAddr| Response::json("{}".to_string()));
        let running = thread::spawn(move || server.run(handler));

        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(response.ends_with("{}"));

        drop(stream);
        // The accept loop ends when the listener is dropped with the thread.
        let _ = running;
    }

    #[test]
    fn converts_parse_failures_into_statuses() {
        assert_eq!(
            error_response(RequestError::Malformed("x")).status,
            Status::BadRequest
        );
        assert_eq!(
            error_response(RequestError::TooLarge("x")).status,
            Status::ContentTooLarge
        );
    }
}
