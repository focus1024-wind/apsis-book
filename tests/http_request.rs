//! Tests for the request parser.
//! 请求解析器的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.
//!
//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。

use apsis_book::http::request::*;

use std::io::Cursor;

#[test]
fn parses_a_get_request() {
    let raw = b"GET /health?verbose=1 HTTP/1.1\r\nHost: localhost\r\nX-Test: a\r\n\r\n";
    let request = read_request(&mut Cursor::new(raw), Limits::default()).unwrap();
    assert_eq!(request.method, Method::Get);
    assert_eq!(request.path, "/health");
    assert_eq!(request.query.as_deref(), Some("verbose=1"));
    assert_eq!(request.header("x-test"), Some("a"));
    assert_eq!(request.header("X-Test"), Some("a"));
    assert_eq!(
        request.query_pairs().get("verbose").map(String::as_str),
        Some("1")
    );
}

#[test]
fn parses_a_post_body() {
    let raw = b"POST /api/tools/x HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello";
    let request = read_request(&mut Cursor::new(raw), Limits::default()).unwrap();
    assert_eq!(request.method, Method::Post);
    assert_eq!(request.body_text(), Some("hello"));
}

#[test]
fn rejects_malformed_input() {
    let cases: &[&[u8]] = &[
        b"GET\r\n\r\n",
        b"GET / HTTP/2.0\r\n\r\n",
        b"GET / HTTP/1.1\r\nbroken\r\n\r\n",
        b"GET / HTTP/1.1\r\nContent-Length: nope\r\n\r\n",
    ];
    for case in cases {
        assert!(read_request(&mut Cursor::new(*case), Limits::default()).is_err());
    }
}

#[test]
fn decodes_percent_escapes() {
    let raw = b"GET /search?q=a%20b+%E2%82%AC HTTP/1.1\r\n\r\n";
    let request = read_request(&mut Cursor::new(raw), Limits::default()).unwrap();
    assert_eq!(
        request.query_pairs().get("q").map(String::as_str),
        Some("a b €")
    );
}

#[test]
fn rejects_an_oversized_body() {
    let raw = b"POST /x HTTP/1.1\r\nContent-Length: 100\r\n\r\n";
    let limits = Limits {
        max_body_bytes: 4,
        ..Limits::default()
    };
    assert!(matches!(
        read_request(&mut Cursor::new(raw), limits),
        Err(RequestError::TooLarge(_))
    ));
}
