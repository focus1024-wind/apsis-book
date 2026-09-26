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

pub mod request;
pub mod response;
pub mod server;

pub use request::{Method, Request};

pub use response::{Response, Status};
pub use server::{Handler, Server, ServerConfig};
