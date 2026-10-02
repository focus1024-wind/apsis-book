//! Tests for the document model and the search tokenizer.
//! 文档模型与搜索分词的测试。
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

use apsis_book::docs::document::*;
use apsis_book::testkit::tokenise;

const SAMPLE: &str = "\
---
id: sgp4
title: SGP4 propagation
chapter: orbit
summary: Analytical propagation of mean elements.
tags: propagation, tle
prerequisites: tle
---

# SGP4

The model propagates [[tle|mean elements]].

## Drag

The BSTAR term.
";

const SAMPLE_ZH: &str = "\
---
title: SGP4 传播
summary: 平均元素的解析传播。
---

# SGP4

该模型传播 [[tle|平均元素]]。

## 阻力

BSTAR 项。
";

/// The layer the sample documents declare in the tests.
/// 测试中的样例文档所声明的层。
const LAYER: &str = "orbit";

/// Parses the English sample under the given identifier.
/// 以给定标识解析英文样例。
fn sample(id: &str) -> Document {
    Document::parse(id, LAYER, Language::English, SAMPLE).expect("the sample must parse")
}

#[test]
fn parses_metadata_and_body() {
    let document = sample("sgp4");
    assert_eq!(document.id, "sgp4");
    assert_eq!(document.title(Language::English), "SGP4 propagation");
    assert_eq!(document.tags, vec!["propagation", "tle"]);
    assert_eq!(document.prerequisites, vec!["tle"]);
    assert_eq!(document.references(), vec!["tle"]);
    assert_eq!(document.english.headings.len(), 2);
    assert_eq!(document.english.headings[1].anchor, "drag");
    assert!(document.english.html.contains("<h1"));
}

#[test]
fn a_repeated_top_level_title_is_dropped_from_the_body() {
    let document = sample("sgp4");
    // The sample body opens with `# SGP4`, which differs from the
    // metadata title `SGP4 propagation`, so it is kept.

    // 样例正文以 `# SGP4` 开头，与元数据标题 `SGP4 propagation` 不同，
    // 因此它被保留。
    assert!(document.english.html.contains("<h1"));
    assert!(
        document
            .english
            .headings
            .iter()
            .any(|heading| heading.text == "SGP4")
    );

    // A body that repeats the metadata title loses the repeated heading.
    // 重复元数据标题的正文会去掉那个重复的标题。
    let repeated = "---\ntitle: SGP4 propagation\n---\n\n# SGP4 propagation\n\nBody.\n";
    let variant = DocumentVariant::parse(repeated).expect("must parse");
    assert!(!variant.html.contains("<h1"));
    assert!(
        !variant
            .headings
            .iter()
            .any(|heading| heading.text == "SGP4 propagation")
    );
    assert!(variant.html.contains("Body."));
}

#[test]
fn a_translation_replaces_the_language_form() {
    let mut document = sample("sgp4");
    assert!(!document.has(Language::Chinese));
    // Before the translation the Chinese reader sees the English form.
    // 译文之前，中文读者看到的是英文形式。
    assert_eq!(document.title(Language::Chinese), "SGP4 propagation");
    document.chinese = Some(DocumentVariant::parse(SAMPLE_ZH).expect("must parse"));
    assert!(document.has(Language::Chinese));
    assert_eq!(document.title(Language::Chinese), "SGP4 传播");
    assert_eq!(document.title(Language::English), "SGP4 propagation");
    // The references of either form are merged.
    // 两种形式的引用被合并。
    assert_eq!(document.references(), vec!["tle"]);
}

#[test]
fn rejects_a_document_without_a_title() {
    // Without a title the document cannot be listed.
    // 没有标题的文档无法被列出。
    assert!(Document::parse("x", LAYER, Language::English, "---\nsummary: s\n---\nbody").is_none());
    assert!(DocumentVariant::parse("---\nsummary: s\n---\nbody").is_none());
}

#[test]
fn language_of_file_uses_the_suffix() {
    assert_eq!(Language::of_file("sgp4"), Language::English);
    assert_eq!(Language::of_file("sgp4.zh"), Language::Chinese);
}

#[test]
fn tree_orders_documents_by_chapter() {
    let documents = vec![
        sample("sgp4"),
        Document::parse(
            "tle",
            LAYER,
            Language::English,
            "---\ntitle: Two line elements\nchapter: orbit\n---\n\nElement sets.\n",
        )
        .expect("must parse"),
    ];
    let tree = Tree::new(documents);
    assert_eq!(tree.len(), 2);
    assert!(tree.document("sgp4").is_some());
    let ids: Vec<&str> = tree
        .ordered()
        .map(|document| document.id.as_str())
        .collect();
    assert_eq!(ids.len(), 2);
}

#[test]
fn a_translated_document_is_indexed_in_both_languages() {
    let mut document = sample("sgp4");
    document.chinese = Some(DocumentVariant::parse(SAMPLE_ZH).expect("must parse"));
    let tree = Tree::new(vec![document]);
    let index = tree.search_index();
    assert_eq!(index.len(), 2);
    let hits = index.search("平均元素", 5);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].language, Language::Chinese);
    // The English body contains "propagates", which the Chinese body
    // does not.

    // 英文正文含有 "propagates"，中文正文没有。
    let hits = index.search("propagates", 5);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].language, Language::English);
}

#[test]
fn tokenise_emits_words_and_han_pairs() {
    assert_eq!(tokenise("mean_motion"), vec!["mean", "motion"]);
    assert_eq!(tokenise("轨道传播"), vec!["轨道", "道传", "传播"]);
    assert_eq!(tokenise("坐标"), vec!["坐标"]);
    // A lone Chinese character produces no token; the search falls back
    // to a substring match for it.

    // 单个中文字符不产生词元；搜索对它回退到子串匹配。
    assert!(tokenise("轨").is_empty());
    // A single Latin character is dropped.
    // 单个拉丁字符被丢弃。
    assert!(tokenise("a").is_empty());
}

#[test]
fn a_chinese_query_matches_a_word_inside_a_sentence() {
    let documents = vec![
        Document::parse(
            "a",
            LAYER,
            Language::English,
            "---\ntitle: Orbits\nsummary: s\n---\n\n本条论述轨道坐标的转换。\n",
        )
        .expect("must parse"),
        Document::parse(
            "b",
            LAYER,
            Language::English,
            "---\ntitle: Time\nsummary: s\n---\n\n这里讨论时间尺度。\n",
        )
        .expect("must parse"),
    ];
    let tree = Tree::new(documents);
    let index = tree.search_index();
    // A two character query matches the pair inside the sentence.
    // 两字查询匹配句子内部的相邻字符对。
    let hits = index.search("坐标", 5);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "a");
    // A one character query has no token and falls back to a substring
    // match.

    // 单字查询没有词元，回退到子串匹配。
    let hits = index.search("轨", 5);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "a");
}

#[test]
fn search_ranks_title_matches_first() {
    let documents = vec![
        Document::parse(
            "a",
            LAYER,
            Language::English,
            "---\ntitle: Redshift\nsummary: x\n---\n\ntext about orbits\n",
        )
        .expect("must parse"),
        Document::parse(
            "b",
            LAYER,
            Language::English,
            "---\ntitle: Orbits\nsummary: y\n---\n\ntext about redshift\n",
        )
        .expect("must parse"),
    ];
    let tree = Tree::new(documents);
    let index = tree.search_index();
    let hits = index.search("redshift orbits", 5);
    assert_eq!(hits.len(), 2);
}

#[test]
fn missing_links_are_reported() {
    let tree = Tree::new(vec![sample("sgp4")]);
    let missing = tree.missing_links();
    assert_eq!(missing, vec![("sgp4".to_string(), "tle".to_string())]);
}

#[test]
fn a_document_is_offered_once_when_its_layer_has_subjects() {
    // The physics layer declares two subjects, so its documents live in
    // them. If a document were also left in the layer's own list, the
    // page would render it twice under one key and stop rendering at
    // all, so no identifier may appear twice in the reading order.

    // 物理层声明了两个子区，因此它的文档住在其中。若某篇文档同时留在层自己的
    // 列表里，页面会在同一个键下渲染它两次而完全停止渲染，因此阅读顺序中任何
    // 标识都不得出现两次。
    let documents = vec![
        Document::parse(
            "time-scales",
            "physics",
            Language::English,
            "---\ntitle: Time scales\nsummary: s\n---\n\nScales.\n",
        )
        .expect("must parse"),
        Document::parse(
            "geodesy-wgs84",
            "physics",
            Language::English,
            "---\ntitle: WGS84\nsummary: s\n---\n\nThe ellipsoid.\n",
        )
        .expect("must parse"),
    ];
    let tree = Tree::new(documents);

    let mut seen: Vec<&String> = tree.reading_order().iter().collect();
    let before = seen.len();
    seen.sort();
    seen.dedup();
    assert_eq!(
        before,
        seen.len(),
        "a document is offered twice: {:?}",
        tree.reading_order()
    );

    // Both documents are still reachable, each under exactly one layer.
    // 两篇文档仍然可达，各自恰好属于一个层。
    let physics = tree
        .groups()
        .iter()
        .flat_map(|group| group.sections.iter())
        .find(|section| section.id == "physics")
        .expect("the physics layer is declared");
    assert!(
        physics.documents.is_empty(),
        "the layer still lists its own documents: {:?}",
        physics.documents
    );
    assert!(physics.subsections.len() >= 2, "the subjects are missing");
}
