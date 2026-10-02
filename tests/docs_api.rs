//! Tests for the interface catalog reader.
//! 接口目录读取器的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。

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
fn an_item_carries_both_languages() {
    // `apsis-rs` writes the doc comment in English and then in Chinese, and
    // the extractor separates the two into the catalogue.

    // `apsis-rs` 把文档注释先写成英文、再写成中文，提取器把两者分开写入目录。
    let item = r#"{"kind":"fn","name":"new","doc":"English.",
            "doc_zh":"中文。","signature":"pub fn new()"}"#;
    let parsed = parse_item(
        &apsis_book::json::parse(item).expect("an item"),
        "apsis-core",
        None,
    )
    .expect("an item");
    assert_eq!(parsed.doc.as_str(), "English.");
    assert_eq!(parsed.doc_zh.as_deref(), Some("中文。"));
}

#[test]
fn an_item_without_a_translation_has_none() {
    // A comment that is English only leaves the Chinese empty, and the
    // reader is then shown the English rather than nothing.

    // 只有英文的注释使中文一侧为空，此时读者看到的是英文，而不是什么都没有。
    let item = r#"{"kind":"fn","name":"new","doc":"English.",
            "signature":"pub fn new()"}"#;
    let parsed = parse_item(
        &apsis_book::json::parse(item).expect("an item"),
        "apsis-core",
        None,
    )
    .expect("an item");
    assert_eq!(parsed.doc_zh, None);
}

#[test]
fn the_committed_catalogue_carries_both_languages_for_every_item() {
    // The translation comes from the source doc comments rather than from a
    // file maintained beside them, so a crate that was never translated shows
    // up here rather than silently reading as English to a Chinese reader.

    // 译文来自源码文档注释，而不是在它们旁边另行维护的文件，因此从未翻译过的
    // crate 会在这里暴露出来，而不是悄悄以英文呈现给中文读者。
    let catalog = apsis_book::docs::load_api_catalog();
    let mut missing = Vec::new();
    for krate in &catalog.crates {
        if krate.summary_zh.is_none() {
            missing.push(format!("{} (crate summary)", krate.name));
        }
        for item in &krate.items {
            if item.doc_zh.is_none() {
                missing.push(format!("{}::{}", krate.name, item.name));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "{} entries have no Chinese: {:?}",
        missing.len(),
        &missing[..missing.len().min(8)]
    );
}
