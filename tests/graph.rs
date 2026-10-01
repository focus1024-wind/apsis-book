//! Tests for the graph model and the crate structure.
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

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
fn crate_structure_follows_the_layer_order() {
    let catalog = apsis_book::docs::load_api_catalog();
    let graph = crate_structure(&catalog);
    assert!(!graph.nodes.is_empty());
    assert!(graph.dangling_edges().is_empty());
    for edge in &graph.edges {
        assert_ne!(edge.source, edge.target);
    }
}
