//! Tests for the graph model and the module structure.
//! 图模型与模块结构的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.
//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。

use apsis_book::docs::load_api_catalog;
use apsis_book::graph::*;

const SAMPLE: &str = r#"{
      "id": "concepts",
      "title": "Concepts",
      "summary": "The concept map.",
      "nodes": [
        {"id": "sgp4", "label": "SGP4", "kind": "concept", "group": "propagation", "summary": "Analytical model.", "document": "sgp4"},
        {"id": "tle", "label": "TLE", "kind": "data", "group": "propagation", "summary": "Element set.", "document": "tle"}
      ],
      "edges": [
        {"source": "sgp4", "target": "tle", "kind": "consumes", "label": "reads"}
      ]
    }"#;

#[test]
fn parses_a_graph() {
    let graph = parse_graph(SAMPLE).unwrap();
    assert_eq!(graph.id, "concepts");
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.node("sgp4").unwrap().kind, NodeKind::Concept);
    assert_eq!(graph.node("tle").unwrap().kind, NodeKind::Data);
    assert_eq!(graph.outgoing("sgp4").count(), 1);
    assert!(graph.dangling_edges().is_empty());
    assert_eq!(graph.groups(), vec!["propagation"]);
}

#[test]
fn reports_dangling_edges() {
    let text = r#"{"id":"x","nodes":[],"edges":[{"source":"a","target":"b","kind":"k"}]}"#;
    let graph = parse_graph(text).unwrap();
    assert_eq!(graph.dangling_edges().len(), 1);
}

#[test]
fn module_structure_follows_the_layer_order() {
    let catalog = load_api_catalog();
    let graph = module_structure(&catalog);
    assert!(!graph.nodes.is_empty());
    assert!(graph.dangling_edges().is_empty());
    for edge in &graph.edges {
        assert_ne!(edge.source, edge.target);
    }
}

/// Every module is a node with a Chinese summary and a link to its page.
///
/// The graph is the reader's map of the crate, so a node that falls back to
/// its English opening line in the Chinese interface, or that leads nowhere,
/// is a hole in it rather than a detail.
///
/// 每个模块都是带中文摘要与自身页面链接的节点。
///
/// 图是读者对该 crate 的地图，因此一个在中文界面回退到英文首句、或指向空处的
/// 节点，都是图上的一个洞，而不是细节。
#[test]
fn every_module_is_a_node_with_a_summary_and_a_link() {
    let catalog = load_api_catalog();
    let graph = module_structure(&catalog);
    let modules: Vec<&Node> = graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Module)
        .collect();
    assert_eq!(modules.len(), catalog.modules.len());
    for node in modules {
        let chinese = node
            .summary_zh
            .as_deref()
            .unwrap_or_else(|| panic!("{} has no Chinese summary", node.id));
        assert_ne!(
            chinese, node.summary,
            "{} repeats its English summary",
            node.id
        );
        assert_eq!(node.api.as_deref(), Some(node.id.as_str()), "{}", node.id);
    }
    let layers = graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Layer)
        .count();
    assert_eq!(layers, 7);
}
