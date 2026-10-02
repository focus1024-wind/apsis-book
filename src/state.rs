//! Shared, immutable service state.
//!
//! Everything the service serves is embedded in the binary and parsed once
//! at startup. The parsed structures are read-only, so a single instance can
//! be shared by every worker thread without a lock.
//!
//! 共享且不可变的服务状态。
//!
//! 服务所提供的一切都嵌在二进制里，并在启动时解析一次。解析出的结构是只读的，
//! 因此单个实例可以由所有工作线程共享而无需加锁。

use crate::docs::{Document, Tree};
use crate::graph::GraphSet;
use crate::tools::Registry;

/// The parsed content served by the service.
///
/// 服务所提供的已解析内容。
pub struct AppState {
    /// Concept documents keyed by identifier, with the navigation tree.
    ///
    /// 以标识为键的概念文档，含导航树。
    pub docs: Tree,
    /// Concept knowledge graph and algorithm flow graphs.
    ///
    /// 概念知识图谱与算法流程图。
    pub graph: GraphSet,
    /// Interface documentation extracted from `apsis-rs`.
    ///
    /// 从 `apsis-rs` 提取的接口文档。
    pub api: crate::docs::ApiCatalog,
    /// The online tool registry.
    ///
    /// 在线工具注册表。
    pub tools: Registry,
}

impl AppState {
    /// Parses the embedded content.
    ///
    /// Content is embedded at compile time, so a parse failure is a build
    /// defect rather than a runtime condition. The fallbacks keep the
    /// process serving the interfaces that do not depend on the failed
    /// source, and the specific document routes report what is missing.
    ///
    /// 解析内嵌内容。
    ///
    /// 内容在编译期嵌入，因此解析失败属于构建缺陷，而不是运行期状况。各处回退
    /// 让进程继续提供不依赖该失败来源的接口，而具体的文档路由会报告缺了什么。
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
    ///
    /// 按标识返回一篇文档。
    pub fn document(&self, id: &str) -> Option<&Document> {
        self.docs.document(id)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
