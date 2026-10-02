//! A minimal HTTP/1.1 server built only from the standard library.
//!
//! The module is split so that each concern can be read on its own:
//!
//! - [`request`] parses an inbound request from a byte stream.
//! - [`response`] builds an outbound response and writes it.
//! - [`server`] accepts connections and dispatches them to a handler on a
//!   bounded pool of worker threads.
//!
//! The server is deliberately synchronous. It handles one request per
//! connection, sends `Connection: close`, and reads the request with a
//! bounded size and a bounded header count so that a slow or hostile peer
//! cannot make the process allocate without limit.
//!
//! 只用标准库写成的极简 HTTP/1.1 服务器。
//!
//! 模块如此拆分，是为了每一件事都能独立阅读：
//!
//! - [`request`] 从字节流解析入站请求。
//! - [`response`] 构造并写出出站响应。
//! - [`server`] 接受连接，并把它们派发给一个规模有界的工作线程池上的处理函数。
//!
//! 服务器刻意做成同步的。每条连接处理一个请求，发送 `Connection: close`，并以
//! 有界的大小与有界的首部数量读取请求，使缓慢或恶意的对端无法让进程无限制地
//! 分配内存。

pub mod request;
pub mod response;
pub mod server;

pub use request::{Method, Request};

pub use response::{Response, Status};
pub use server::{Handler, Server, ServerConfig};
