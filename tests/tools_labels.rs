//! Tests for the tool result label table.
//! 工具结果标签表的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。

use apsis_book::tools::labels::*;

#[test]
fn a_known_label_is_translated() {
    let text = label("Semi-major axis");
    assert_eq!(text.en, "Semi-major axis");
    assert_eq!(text.zh, "半长轴");
}

#[test]
fn an_unknown_label_is_shown_as_it_is() {
    let text = label("position.x");
    assert_eq!(text.en, "position.x");
    assert_eq!(text.zh, "position.x");
}

#[test]
fn a_repeated_lookup_returns_the_same_static_text() {
    let first = label("not-a-known-label");
    let second = label("not-a-known-label");
    assert_eq!(first.zh.as_ptr(), second.zh.as_ptr());
}
