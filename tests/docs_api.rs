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
use apsis_book::docs::load_api_catalog;
use apsis_book::testkit::parse_item;

const SAMPLE: &str = r#"{
      "generated_from": ["crates/apsis/src/orbit/apsis_sgp4"],
      "modules": [
        {
          "name": "apsis_sgp4",
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
    assert_eq!(catalog.modules.len(), 1);
    assert_eq!(catalog.item_count(), 2);
    let module = catalog.module("apsis_sgp4").unwrap();
    assert_eq!(module.layer, "orbit");
    assert_eq!(module.items_of_kind("fn").len(), 1);
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
        "apsis_core",
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
        "apsis_core",
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
    for module in &catalog.modules {
        if module.summary_zh.is_none() {
            missing.push(format!("{} (module summary)", module.name));
        }
        for item in &module.items {
            if item.doc_zh.is_none() {
                missing.push(format!("{}::{}", module.name, item.name));
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

/// Every layer of the crate must be represented in the committed catalog.
///
/// The service documents all seven layers and the dependency enables every
/// one of them, so a catalog that is short of a layer is a build defect
/// rather than a smaller feature set.
///
/// 该 crate 的每一层都必须在已提交的目录中出现。
///
/// 服务为全部七层编写文档，依赖也启用了每一层，因此缺少某一层的目录属于构建缺陷，
/// 而不是更小的特性集。
#[test]
fn every_layer_has_modules() {
    let catalog = load_api_catalog();
    for layer in [
        "foundation",
        "physics",
        "orbit",
        "rf",
        "network",
        "ops",
        "sim",
    ] {
        let count = catalog
            .modules
            .iter()
            .filter(|module| module.layer == layer)
            .count();
        assert!(count > 0, "no module in the {layer} layer");
    }
}
