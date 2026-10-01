//! Tests for the listener and the worker pool.
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.
use apsis_book::http::request::{Request, RequestError};
use apsis_book::http::response::{Response, Status};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;

use apsis_book::http::server::*;
use apsis_book::testkit::error_response;

use std::io::{Read, Write};

#[test]
fn serves_a_request_over_a_real_socket() {
    let server = Server::bind_address(
        "127.0.0.1:0"
            .parse()
            .expect("the literal address must parse"),
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
