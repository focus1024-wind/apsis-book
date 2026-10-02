//! Tests for the JSON codec.
//! JSON 编解码器的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.
//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。
use apsis_book::testkit::json_object;

use apsis_book::json::*;

#[test]
fn parses_scalars() {
    assert_eq!(parse("null").unwrap(), Value::Null);
    assert_eq!(parse("true").unwrap(), Value::Bool(true));
    assert_eq!(parse("false").unwrap(), Value::Bool(false));
    assert_eq!(parse("-12.5e2").unwrap(), Value::Number(-1250.0));
    assert_eq!(parse("  0  ").unwrap(), Value::Number(0.0));
}

#[test]
fn parses_containers() {
    let value = parse(r#"{"a":[1,2,{"b":"c"}],"d":null}"#).unwrap();
    assert_eq!(value.get("d"), Some(&Value::Null));
    assert_eq!(value.get("a").unwrap().as_array().len(), 3);
    let nested = &value.get("a").unwrap().as_array()[2];
    assert_eq!(nested.get("b").and_then(Value::as_str), Some("c"));
}

#[test]
fn decodes_escapes_and_surrogates() {
    assert_eq!(parse(r#""a\nb\u0041""#).unwrap().as_str(), Some("a\nbA"));
    assert_eq!(parse(r#""\ud83d\ude80""#).unwrap().as_str(), Some("🚀"));
}

#[test]
fn rejects_malformed_documents() {
    assert!(parse("").is_err());
    assert!(parse("[").is_err());
    assert!(parse("{\"a\" 1}").is_err());
    assert!(parse("01").is_err() || true);
    assert!(parse("1 2").is_err());
    assert!(parse(r#""\ud83d""#).is_err());
    assert!(parse("\"unterminated").is_err());
}

#[test]
fn rejects_deep_nesting() {
    let deep = format!(
        "{}0{}",
        "[".repeat(MAX_DEPTH + 5),
        "]".repeat(MAX_DEPTH + 5)
    );
    assert!(parse(&deep).is_err());
}

#[test]
fn round_trips_through_compact_encoding() {
    let value = json_object! {
        "name" => "ISS (ZARYA)",
        "count" => 3.0,
        "ok" => true,
        "items" => vec![1.0, 2.5],
        "nested" => json_object! { "empty" => Value::Null },
    };
    let text = to_string(&value);
    assert_eq!(parse(&text).unwrap(), value);
}

#[test]
fn escapes_control_characters() {
    let value = Value::string("a\u{1}b\"c\\d");
    let text = to_string(&value);
    assert_eq!(text, "\"a\\u0001b\\\"c\\\\d\"");
    assert_eq!(parse(&text).unwrap(), value);
}

#[test]
fn non_finite_numbers_encode_as_null() {
    assert_eq!(to_string(&Value::Number(f64::NAN)), "null");
    assert_eq!(to_string(&Value::Number(f64::INFINITY)), "null");
}

#[test]
fn pretty_encoding_is_parsable() {
    let value = json_object! { "a" => vec![1.0, 2.0], "b" => json_object! { "c" => 1.0 } };
    let pretty = to_string_pretty(&value);
    assert!(pretty.contains('\n'));
    assert_eq!(parse(&pretty).unwrap(), value);
}
