//! Tests for access to the embedded frontend assets.
//! 对内嵌前端资源的访问的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。

use apsis_book::embedded::*;

#[test]
fn content_type_covers_the_built_assets() {
    assert_eq!(content_type("index.html"), "text/html; charset=utf-8");
    assert_eq!(
        content_type("assets/index-abc.js"),
        "text/javascript; charset=utf-8"
    );
    assert_eq!(
        content_type("assets/index-abc.css"),
        "text/css; charset=utf-8"
    );
    assert_eq!(content_type("logo.svg"), "image/svg+xml");
    assert_eq!(content_type("data.bin"), "application/octet-stream");
}

#[test]
fn placeholder_is_html() {
    let page = std::str::from_utf8(placeholder()).unwrap();
    assert!(page.starts_with("<!doctype html>"));
}

#[test]
fn lookup_of_an_unknown_path_is_empty() {
    assert!(get("definitely/not/there.js").is_none());
}
