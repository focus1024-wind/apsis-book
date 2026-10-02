//! Tests for the bilingual strings.
//! 双语字符串的测试。
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

use apsis_book::i18n::*;

use apsis_book::json;

#[test]
fn encodes_both_languages() {
    let value = Text::new("轨道", "orbit").json();
    assert_eq!(value.get("zh").unwrap().as_str(), Some("轨道"));
    assert_eq!(value.get("en").unwrap().as_str(), Some("orbit"));
    let encoded = json::to_string(&value);
    assert!(json::parse(&encoded).is_ok());
}
