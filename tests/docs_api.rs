//! Tests for the interface catalog reader.
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

use apsis_book::docs::api::*;
use apsis_book::testkit::parse_item;

const SAMPLE: &str = r#"{
      "generated_from": ["crates/orbit/apsis-sgp4/src/lib.rs"],
      "crates": [
        {
          "name": "apsis-sgp4",
          "layer": "orbit",
          "summary": "The SGP4 and SDP4 analytical orbit propagator.",
          "items": [
            {"kind": "struct", "name": "Satellite", "signature": "pub struct Satellite { /* private */ }", "doc": "A prepared satellite.", "associated": false},
            {"kind": "fn", "name": "propagate_minutes", "signature": "pub fn propagate_minutes(&self, minutes: f64) -> Result<State, PropagationError>", "doc": "Propagates by minutes.", "associated": true}
          ]
        }
      ]
    }"#;

#[test]
fn parses_the_catalog() {
    let catalog = parse(SAMPLE);
    assert_eq!(catalog.crates.len(), 1);
    assert_eq!(catalog.item_count(), 2);
    let krate = catalog.krate("apsis-sgp4").unwrap();
    assert_eq!(krate.layer, "orbit");
    assert_eq!(krate.items_of_kind("fn").len(), 1);
    let (_, item) = catalog.find_item("propagate_minutes").unwrap();
    assert!(item.is_function());
    assert!(item.associated);
}

#[test]
fn malformed_input_yields_an_empty_catalog() {
    assert!(parse("not json").is_empty());
    assert!(parse("{}").is_empty());
}

#[test]
fn a_translation_filed_under_the_crate_beats_the_bare_name() {
    // `new` means something different in each crate, so a translation
    // filed under the name alone would be shown against the wrong
    // documentation.
    let item = r#"{"kind":"fn","name":"new","doc":"English.",
            "signature":"pub fn new()"}"#;
    let texts = r#"{
            "new":"裸译。",
            "apsis-core::new":"属于 apsis-core 的译文。"}"#;
    let parsed = parse_item(
        &apsis_book::json::parse(item).expect("an item"),
        Some(&apsis_book::json::parse(texts).expect("the translations")),
        "apsis-core",
        None,
    )
    .expect("an item");
    assert_eq!(parsed.doc_zh.as_deref(), Some("属于 apsis-core 的译文。"));
}

#[test]
fn a_translation_filed_under_the_owner_separates_two_items_of_one_crate() {
    // Two types of one crate each have a `new`; the owner tells them
    // apart.
    let item = r#"{"kind":"fn","name":"new","doc":"English.",
            "signature":"pub fn new()"}"#;
    let texts = r#"{"Inertial::new":"惯性系的新建。"}"#;
    let parsed = parse_item(
        &apsis_book::json::parse(item).expect("an item"),
        Some(&apsis_book::json::parse(texts).expect("the translations")),
        "apsis-coord",
        Some("Inertial"),
    )
    .expect("an item");
    assert_eq!(parsed.doc_zh.as_deref(), Some("惯性系的新建。"));
}
