//! Tests for the tool handlers.
//! 工具处理函数的测试。
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

use apsis_book::tools::ToolError;
use apsis_book::tools::handlers::*;

use apsis_book::json::Value;
use apsis_book::tools::{Params, Registry};

/// Runs a tool with the given parameters.
/// 以给定参数运行一个工具。
fn call(id: &str, values: Value) -> Result<Value, ToolError> {
    let registry = Registry::new();
    let tool = registry.get(id).unwrap();
    let params = Params::new(tool, &values).unwrap();
    run(id, &params)
}

/// Returns the English text of a label, which is a bilingual object.
/// 返回某个标签的英文文本，该标签是一个双语对象。
fn label_english(value: &Value) -> &str {
    value.get("en").and_then(Value::as_str).unwrap_or("")
}

/// Returns the numeric value of the scalar with an English label.
/// 返回英文标签为给定值的那个标量的数值。
fn scalar_number(value: &Value, label: &str) -> f64 {
    value
        .get("scalars")
        .unwrap()
        .as_array()
        .iter()
        .find(|scalar| scalar.get("label").map(label_english) == Some(label))
        .unwrap()
        .get("value")
        .unwrap()
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

const TLE: &str = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\n2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";

#[test]
fn parse_tle_reports_the_fields() {
    let value = call("parse-tle", json_object! { "tle" => TLE }).unwrap();
    assert_eq!(value.get("kind").unwrap().as_str(), Some("element-set"));
    let rows = value.get("table").unwrap().get("rows").unwrap().as_array();
    assert_eq!(rows.len(), 10);
}

#[test]
fn parse_tle_rejects_a_bad_checksum() {
    let broken = TLE.replace("2927", "2928");
    let error = call("parse-tle", json_object! { "tle" => broken }).unwrap_err();
    assert_eq!(error.parameter, Some("tle"));
}

#[test]
fn sgp4_propagates_the_sample() {
    let value = call(
        "sgp4-propagate",
        json_object! { "tle" => TLE, "minutes" => 0.0 },
    )
    .unwrap();
    let radius = scalar_number(&value, "Radius");
    assert!((6_600.0..7_200.0).contains(&radius));
}

#[test]
fn sgp4_track_returns_samples_and_series() {
    let value = call(
            "sgp4-track",
            json_object! { "tle" => TLE, "start_minutes" => 0.0, "end_minutes" => 10.0, "step_minutes" => 1.0 },
        )
        .unwrap();
    let rows = value.get("table").unwrap().get("rows").unwrap().as_array();
    assert_eq!(rows.len(), 11);
    assert_eq!(value.get("series").unwrap().as_array().len(), 2);
    assert!(value.get("axis").is_some());
}

#[test]
fn sgp4_track_rejects_a_range_that_is_too_large() {
    let error = call(
            "sgp4-track",
            json_object! { "tle" => TLE, "start_minutes" => 0.0, "end_minutes" => 100000.0, "step_minutes" => 0.001 },
        )
        .unwrap_err();
    assert_eq!(error.parameter, Some("step_minutes"));
}

#[test]
fn osculating_comparison_lists_six_rows() {
    let value = call("orbit-osculating", json_object! { "tle" => TLE }).unwrap();
    let rows = value.get("table").unwrap().get("rows").unwrap().as_array();
    assert_eq!(rows.len(), 6);
}

#[test]
fn kepler_conversion_returns_a_state() {
    let value = call("kepler-state", json_object! {}).unwrap();
    assert_eq!(value.get("kind").unwrap().as_str(), Some("state-vector"));
}

#[test]
fn kepler_conversion_rejects_a_non_elliptical_orbit() {
    // An eccentricity of one is a parabola; the conversion requires an
    // ellipse, so the tool reports the condition without naming a
    // parameter.

    // 偏心率为一即抛物线；该转换要求椭圆，因此工具报告该条件而不指名某个参数。
    let error = call(
        "kepler-state",
        json_object! { "eccentricity" => 1.0, "semi_major_axis" => 7000.0 },
    )
    .unwrap_err();
    assert!(error.parameter.is_none());
}

#[test]
fn time_conversion_lists_every_scale() {
    let value = call(
        "time-convert",
        json_object! { "input" => "calendar", "year" => 2024.0, "month" => 6.0, "day" => 15.0 },
    )
    .unwrap();
    let rows = value.get("table").unwrap().get("rows").unwrap().as_array();
    assert_eq!(rows.len(), 4);
}

#[test]
fn time_conversion_rejects_a_date_outside_the_leap_table() {
    let error = call(
        "time-convert",
        json_object! { "input" => "calendar", "year" => 1960.0, "month" => 1.0, "day" => 1.0 },
    )
    .unwrap_err();
    assert!(error.message.contains("1972"));
}

#[test]
fn geodetic_conversion_matches_the_equatorial_radius() {
    let value = call(
        "geodetic-ecef",
        json_object! { "latitude" => 0.0, "longitude" => 0.0, "height" => 0.0 },
    )
    .unwrap();
    let x = scalar_number(&value, "X");
    assert!((x - 6_378_137.0).abs() < 1e-3);
}

#[test]
fn frame_rotation_preserves_the_radius() {
    let value = call(
            "frame-transform",
            json_object! { "x" => 6871000.0, "y" => 1234000.0, "z" => 500000.0, "earth_rotation" => 0.3 },
        )
        .unwrap();
    let change = scalar_number(&value, "Radius change");
    assert!(change.abs() < 1e-6);
}

#[test]
fn hohmann_transfer_reports_two_burns() {
    let value = call("hohmann-transfer", json_object! {}).unwrap();
    assert_eq!(value.get("kind").unwrap().as_str(), Some("maneuver"));
    let rows = value.get("table").unwrap().get("rows").unwrap().as_array();
    assert_eq!(rows.len(), 2);
}

#[test]
fn hohmann_transfer_rejects_equal_radii() {
    let error = call(
        "hohmann-transfer",
        json_object! { "from_radius" => 7000.0, "to_radius" => 7000.0 },
    )
    .unwrap_err();
    assert_eq!(error.parameter, Some("to_radius"));
}

#[test]
fn unknown_tools_are_rejected() {
    assert!(
        run(
            "nope",
            &Params::new(Registry::new().get("parse-tle").unwrap(), &json_object! {}).unwrap()
        )
        .is_err()
    );
}

#[test]
fn encoded_results_are_parsable_json() {
    let value = call("sgp4-propagate", json_object! { "tle" => TLE }).unwrap();
    assert!(apsis_book::json::parse(&apsis_book::json::to_string(&value)).is_ok());
}
