//! Shared, immutable service state.
//!
//! Everything the service serves is embedded in the binary and parsed once
//! at startup. The parsed structures are read-only, so a single instance can
//! be shared by every worker thread without a lock.

use crate::docs::{Document, Tree};
use crate::graph::GraphSet;
use crate::tools::Registry;

/// The parsed content served by the service.
pub struct AppState {
    /// Concept documents keyed by identifier, with the navigation tree.
    pub docs: Tree,
    /// Concept knowledge graph and algorithm flow graphs.
    pub graph: GraphSet,
    /// Interface documentation extracted from `apsis-rs`.
    pub api: crate::docs::ApiCatalog,
    /// The online tool registry.
    pub tools: Registry,
}

impl AppState {
    /// Parses the embedded content.
    ///
    /// Content is embedded at compile time, so a parse failure is a build
    /// defect rather than a runtime condition. The fallbacks keep the
    /// process serving the interfaces that do not depend on the failed
    /// source, and the specific document routes report what is missing.
    pub fn new() -> Self {
        let documents = crate::docs::load_documents();
        let graph = crate::graph::load_graph();
        let api = crate::docs::load_api_catalog();
        Self {
            docs: Tree::new(documents),
            graph,
            api,
            tools: Registry::new(),
        }
    }

    /// Returns a document by identifier.
    pub fn document(&self, id: &str) -> Option<&Document> {
        self.docs.document(id)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
