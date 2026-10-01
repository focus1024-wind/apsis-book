//! Tests for the bilingual strings.
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

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
