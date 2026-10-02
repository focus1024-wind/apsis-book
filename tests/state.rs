//! Tests for the parsed content shared by the workers.
//! 各工作线程共享的已解析内容的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。

use apsis_book::state::*;

#[test]
fn embedded_content_parses_and_is_linked() {
    let state = AppState::new();
    assert!(
        state.docs.len() >= 10,
        "the concept set should be populated"
    );
    assert!(
        !state.graph.concepts.nodes.is_empty(),
        "the graph should be populated"
    );
    assert!(
        !state.graph.flows.is_empty(),
        "the flow graphs should be populated"
    );
    assert!(
        !state.tools.entries().is_empty(),
        "tools should be registered"
    );
    assert!(
        !state.api.crates.is_empty(),
        "the interface catalog should be populated"
    );
    assert!(
        state.docs.missing_links().is_empty(),
        "every cross reference should resolve"
    );
    for graph in std::iter::once(&state.graph.concepts).chain(state.graph.flows.iter()) {
        assert!(
            graph.dangling_edges().is_empty(),
            "every edge of {} should connect two nodes",
            graph.id
        );
    }
}

#[test]
fn graph_nodes_point_at_documents_and_interface_items() {
    let state = AppState::new();
    let graphs = std::iter::once(&state.graph.concepts).chain(state.graph.flows.iter());
    for graph in graphs {
        for node in &graph.nodes {
            if let Some(id) = &node.document {
                assert!(
                    state.docs.document(id).is_some(),
                    "node {} of {} names document {id}, which does not exist",
                    node.id,
                    graph.id
                );
            }
            if let Some(name) = &node.api {
                assert!(
                    state.api.find_item(name).is_some(),
                    "node {} of {} names interface item {name}, which does not exist",
                    node.id,
                    graph.id
                );
            }
        }
    }
}

#[test]
fn tool_api_items_resolve() {
    let state = AppState::new();
    for tool in state.tools.entries() {
        for name in tool.api_items {
            assert!(
                state.api.find_item(name).is_some(),
                "tool {} names interface item {name}, which does not exist",
                tool.id
            );
        }
    }
}
