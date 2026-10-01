//! Tests for the response writer.
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

use apsis_book::http::response::*;

#[test]
fn writes_status_and_length() {
    let response = Response::json("{\"a\":1}".to_string());
    let text = String::from_utf8(response.to_bytes()).unwrap();
    assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(text.contains("Content-Type: application/json; charset=utf-8\r\n"));
    assert!(text.contains("Content-Length: 7\r\n"));
    assert!(text.ends_with("\r\n\r\n{\"a\":1}"));
}

#[test]
fn head_response_keeps_the_length_but_drops_the_body() {
    let response = Response::json("{}".to_string());
    let mut output = Vec::new();
    response.write_to(&mut output, true).unwrap();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("Content-Length: 2\r\n"));
    assert!(text.ends_with("\r\n\r\n"));
}

#[test]
fn empty_body_has_no_content_type() {
    let response = Response::empty(Status::NoContent);
    let text = String::from_utf8(response.to_bytes()).unwrap();
    // The response carries `X-Content-Type-Options`, so the assertion is
    // on the leading header line rather than on the substring.
    assert!(!text.contains("\r\nContent-Type:"));
    assert!(text.contains("Content-Length: 0\r\n"));
}
