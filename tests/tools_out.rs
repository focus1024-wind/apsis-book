//! Tests for the tool result shape.
//! 工具结果形状的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.
//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。
use apsis_book::json::Value;

use apsis_book::tools::labels;
use apsis_book::tools::out::*;

/// The first cell of the first row of a shaped response.
/// 某个已整形响应中第一行第一格的单元格。
fn first_cell(value: &Value) -> Value {
    let rows = value
        .get("table")
        .and_then(|table| table.get("rows"))
        .expect("the response carries a table")
        .as_array();
    rows.first()
        .and_then(|row| row.as_array().first())
        .expect("the table carries a row")
        .clone()
}

#[test]
fn a_text_cell_is_shown_in_either_language() {
    // The meaning of a field is a fixed phrase, so it is looked up rather
    // than written twice.

    // 字段的含义是固定短语，因此走查表而不是写两遍。
    let mut table = Table::new(vec![Column::text("meaning", "Meaning")]);
    table.push(vec![Value::from("Mean motion")]);
    let value = table_response(
        "kind",
        "Title",
        Vec::new(),
        &table,
        Vec::new(),
        vec![labels::label("Mean motion")],
    );
    let cell = first_cell(&value);
    assert_eq!(cell.get("zh").and_then(Value::as_str), Some("平均运动"));
    assert_eq!(cell.get("en").and_then(Value::as_str), Some("Mean motion"));
}

#[test]
fn an_identifier_is_not_set_as_a_formula() {
    // `mean_motion` is a field name. Set as a formula it would come out as
    // separate letters with a subscript, which is not the field name.

    // `mean_motion` 是一个字段名。当成公式排版会变成逐字母加下标，那就不是
    // 字段名了。
    let mut table = Table::new(vec![Column::text("field", "Field")]);
    table.push(vec![Value::from("mean_motion")]);
    let value = table_response("kind", "Title", Vec::new(), &table, Vec::new(), Vec::new());
    let cell = first_cell(&value);
    assert!(
        cell.get("html").is_none(),
        "an identifier should not be rendered: {cell:?}"
    );
}

#[test]
fn a_unit_with_a_power_is_rendered() {
    // `rev/day^2` is a unit, and the power is what a reader needs raised.
    // `rev/day^2` 是一个单位，读者需要看到的是被抬起的幂次。
    let mut table = Table::new(vec![Column::text("unit", "Unit")]);
    table.push(vec![Value::from("rev/day^2")]);
    let value = table_response("kind", "Title", Vec::new(), &table, Vec::new(), Vec::new());
    let cell = first_cell(&value);
    let html = cell.get("html").and_then(Value::as_str).unwrap_or_default();
    assert!(html.contains("<msup>"), "the power was not raised: {html}");
}

#[test]
fn table_response_carries_the_shape() {
    let mut table = Table::new(vec![Column::number("a", "A", "km")]);
    table.push(vec![Value::from(1.0)]);
    let value = table_response(
        "table",
        "T",
        vec![Scalar::plain("x", 1)],
        &table,
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(value.get("kind").unwrap().as_str(), Some("table"));
    assert_eq!(
        value
            .get("table")
            .unwrap()
            .get("rows")
            .unwrap()
            .as_array()
            .len(),
        1
    );
    assert_eq!(value.get("scalars").unwrap().as_array().len(), 1);
}

#[test]
fn conversions_round_trip() {
    assert!((degrees(radians(30.0)) - 30.0).abs() < 1e-12);
}
