//! Connection acceptance and dispatch to a bounded worker pool.
//!
//! 连接的接受，以及向有界工作线程池的派发。

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
///
/// 一个请求处理函数。
///
/// 处理函数接收请求与对端地址，返回要发送的响应。它不得长时间阻塞在别的处理函数
/// 可能持有的共享状态上，因为并发处理的请求数量是有界的。
pub trait Handler: Send + Sync + 'static {
    /// Produces the response for one request.
    /// 为一个请求产生响应。
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
///
/// 服务器的可调参数。
#[derive(Debug, Clone, Copy)]
pub struct ServerConfig {
    /// Number of worker threads.
    /// 工作线程数。
    pub workers: usize,
    /// Maximum number of connections waiting for a worker.
    /// 等待工作线程的连接数上限。
    pub backlog: usize,
    /// Read and write timeout applied to every connection.
    /// 施加于每条连接的读写超时。
    pub timeout: Duration,
    /// Request size limits.
    /// 请求大小上限。
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
///
/// 一个运行中的 HTTP 服务器。
pub struct Server {
    listener: TcpListener,
    config: ServerConfig,
}

impl Server {
    /// Binds a server to a socket address.
    ///
    /// This is the form the command line uses once the host and the port
    /// have been resolved, so that a name is resolved once rather than left
    /// to the listener.
    ///
    /// 把服务器绑定到一个套接字地址。
    ///
    /// 命令行在主机与端口都已解析之后使用这种形式，使名字只解析一次，而不留给
    /// 监听器处理。
    pub fn bind_address(address: SocketAddr, config: ServerConfig) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        Ok(Self { listener, config })
    }

    /// Returns the address the server is bound to.
    ///
    /// This is useful when the configured port is zero and the operating
    /// system assigned one.
    ///
    /// 返回服务器所绑定的地址。
    ///
    /// 当配置的端口为零而由操作系统分配了一个时，这很有用。
    pub fn local_address(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Runs the accept loop until the listener fails.
    ///
    /// Every accepted connection is handed to a worker thread from a fixed
    /// size pool. When all workers are busy a connection waits in the queue,
    /// so memory use stays bounded by the pool size and the queue capacity.
    ///
    /// 运行接受循环，直到监听器失败。
    ///
    /// 每条被接受的连接都交给固定规模线程池中的一个工作线程。所有工作线程都忙时，
    /// 连接在队列中等待，因此内存用量始终受线程池规模与队列容量限制。
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

                    // 线程池断开意味着进程正在关闭；丢弃该流即关闭它。
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
///
/// 从队列中取出并处理请求，直到所有发送端都被丢弃。
fn worker_loop<H: Handler>(
    receiver: &Mutex<mpsc::Receiver<TcpStream>>,
    handler: &H,
    config: ServerConfig,
) {
    loop {
        // The lock is released before the request is served, so workers do
        // not serialise on the queue.

        // 锁在请求被处理之前释放，因此各工作线程不会在队列上串行。
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
///
/// 从一条连接读取一个请求并写出响应。
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
///
/// 把一次请求解析失败转为一个响应。
///
/// 响应体描述失败的类别而不复述输入，使畸形的请求无法被用来反射任意文本。
pub(crate) fn error_response(error: RequestError) -> Response {
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
///
/// 调用处理函数，把 panic 转为服务器错误响应。
///
/// 处理函数是意外状况唯一可能出现的地方，因此也是唯一需要这道防线的地方。捕获该
/// panic 使一个坏请求不会拖垮一个工作线程。
fn dispatch<H: Handler>(handler: &H, request: &Request, peer: SocketAddr) -> Response {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        handler.handle(request, peer)
    }))
    .unwrap_or_else(|_| Response::error(Status::InternalServerError, "internal server error"))
}
