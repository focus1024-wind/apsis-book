//! Tests for the tool registry.
//! 工具注册表的测试。
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

use apsis_book::tools::*;

#[test]
fn registry_is_populated() {
    let registry = Registry::new();
    assert!(registry.entries().len() >= 7);
    assert!(registry.get("parse-tle").is_some());
    assert!(registry.get("sgp4-propagate").is_some());
    assert!(registry.get("missing").is_none());
}

#[test]
fn registry_json_lists_tools_and_categories() {
    let value = registry_json(&Registry::new());
    assert!(!value.get("tools").unwrap().as_array().is_empty());
    assert!(!value.get("categories").unwrap().as_array().is_empty());
}

#[test]
fn number_parameters_honour_defaults_and_bounds() {
    let registry = Registry::new();
    let tool = registry.get("sgp4-propagate").unwrap();
    let values = json_object! {};
    let params = Params::new(tool, &values).unwrap();
    assert_eq!(params.number("minutes").unwrap(), 0.0);
}

#[test]
fn rejects_out_of_range_values() {
    let registry = Registry::new();
    let tool = registry.get("sgp4-propagate").unwrap();
    let values = json_object! { "minutes" => 5_000_000.0 };
    let params = Params::new(tool, &values).unwrap();
    let error = params.number("minutes").unwrap_err();
    assert_eq!(error.parameter, Some("minutes"));
}

#[test]
fn select_parameters_reject_unknown_choices() {
    let registry = Registry::new();
    let tool = registry.get("sgp4-propagate").unwrap();
    let values = json_object! { "model" => "wgs99" };
    let params = Params::new(tool, &values).unwrap();
    assert!(params.choice("model").is_err());
    let values = json_object! { "model" => "WGS84" };
    let params = Params::new(tool, &values).unwrap();
    assert_eq!(params.choice("model").unwrap(), "wgs84");
}

#[test]
fn string_numbers_are_accepted() {
    let registry = Registry::new();
    let tool = registry.get("sgp4-propagate").unwrap();
    let values = json_object! { "minutes" => "12.5" };
    let params = Params::new(tool, &values).unwrap();
    assert_eq!(params.number("minutes").unwrap(), 12.5);
}
