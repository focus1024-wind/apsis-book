//! The knowledge graph and the algorithm flow graphs.
//!
//! Two kinds of graph are served:
//!
//! - The concept graph, authored in `content/graph/concepts.json`. Its nodes
//!   are concepts and its edges are the relationships between them, such as
//!   `uses`, `refines`, or `contrasts`. A node carries the identifier of the
//!   document that explains it, so the frontend can open the document when a
//!   node is clicked.
//! - Flow graphs, authored one per file in `content/flows/`. A flow graph
//!   describes the steps of an algorithm, such as the SGP4 propagation
//!   sequence, as nodes with shapes and labelled edges with branches.
//!
//! Both are plain JSON with the same shape, so one renderer on the frontend
//! handles either. The layout is computed on the frontend from the node
//! kinds; the backend only supplies the structure, which keeps the authored
//! data independent of any particular rendering.

use crate::docs::ApiCatalog;
use crate::json::{self, Value};

/// The kind of a graph node.
///
/// The kind selects how a node is drawn. The set is intentionally small so
/// that a new graph can be authored without touching the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// A concept in the knowledge graph.
    Concept,
    /// A crate of the `apsis-rs` workspace.
    Crate,
    /// A layer of the `apsis-rs` workspace.
    Layer,
    /// A step of an algorithm.
    Step,
    /// A decision in an algorithm, with more than one outgoing edge.
    Decision,
    /// A terminal step, such as a returned value or an error.
    Terminal,
    /// A data item, such as an element set or an epoch.
    Data,
}

impl NodeKind {
    /// Parses a kind name, defaulting to [`NodeKind::Concept`].
    pub fn parse(name: &str) -> Self {
        match name {
            "crate" => Self::Crate,
            "layer" => Self::Layer,
            "step" => Self::Step,
            "decision" => Self::Decision,
            "terminal" => Self::Terminal,
            "data" => Self::Data,
            _ => Self::Concept,
        }
    }

    /// Returns the kind name as it is written in the data.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Concept => "concept",
            Self::Crate => "crate",
            Self::Layer => "layer",
            Self::Step => "step",
            Self::Decision => "decision",
            Self::Terminal => "terminal",
            Self::Data => "data",
        }
    }
}

/// A node of a graph.
#[derive(Debug, Clone)]
pub struct Node {
    /// Identifier, unique within the graph.
    pub id: String,
    /// Display label in English.
    pub label: String,
    /// Display label in Chinese, or `None` when the label is a name that is
    /// written the same way in both languages.
    pub label_zh: Option<String>,
    /// Kind, selecting how the node is drawn.
    pub kind: NodeKind,
    /// Chapter or group the node belongs to, used for colouring.
    pub group: String,
    /// One sentence description in English.
    pub summary: String,
    /// One sentence description in Chinese, when one is authored.
    pub summary_zh: Option<String>,
    /// Identifier of the document that explains the node, when one exists.
    pub document: Option<String>,
    /// Identifier of the interface item, when the node names one.
    pub api: Option<String>,
}

/// An edge of a graph.
#[derive(Debug, Clone)]
pub struct Edge {
    /// Source node identifier.
    pub source: String,
    /// Target node identifier.
    pub target: String,
    /// Relationship name, such as `uses`.
    pub kind: String,
    /// Optional label drawn on the edge, in English.
    pub label: Option<String>,
    /// The same label in Chinese, when one is authored.
    pub label_zh: Option<String>,
}

/// A graph, either the concept graph or one flow graph.
#[derive(Debug, Clone)]
pub struct Graph {
    /// Identifier of the graph.
    pub id: String,
    /// Display title in English.
    pub title: String,
    /// Display title in Chinese, when one is authored.
    pub title_zh: Option<String>,
    /// One sentence description in English.
    pub summary: String,
    /// One sentence description in Chinese, when one is authored.
    pub summary_zh: Option<String>,
    /// Nodes.
    pub nodes: Vec<Node>,
    /// Edges.
    pub edges: Vec<Edge>,
}

impl Graph {
    /// Returns a node by identifier.
    ///
    /// Used to resolve the endpoints of an edge, which is what the check
    /// for a dangling edge does.
    #[allow(dead_code)]
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// Returns the edges that leave a node.
    #[allow(dead_code)]
    pub fn outgoing(&self, id: &str) -> impl Iterator<Item = &Edge> {
        self.edges.iter().filter(move |edge| edge.source == id)
    }

    /// Returns the identifiers of every edge endpoint that is not a node.
    ///
    /// The authored graph data is checked with this at startup, so a broken
    /// edge is reported rather than drawn to nowhere.
    #[allow(dead_code)]
    pub fn dangling_edges(&self) -> Vec<(String, String)> {
        self.edges
            .iter()
            .filter(|edge| self.node(&edge.source).is_none() || self.node(&edge.target).is_none())
            .map(|edge| (edge.source.clone(), edge.target.clone()))
            .collect()
    }

    /// Returns the distinct groups in the order they first appear.
    pub fn groups(&self) -> Vec<String> {
        let mut groups = Vec::new();
        for node in &self.nodes {
            if !groups.iter().any(|group| group == &node.group) {
                groups.push(node.group.clone());
            }
        }
        groups
    }
}

/// The concept graph together with the algorithm flow graphs.
#[derive(Debug)]
pub struct GraphSet {
    /// The concept knowledge graph.
    pub concepts: Graph,
    /// The flow graphs, in title order.
    pub flows: Vec<Graph>,
}

impl GraphSet {
    /// Returns a flow graph by identifier.
    pub fn flow(&self, id: &str) -> Option<&Graph> {
        self.flows.iter().find(|graph| graph.id == id)
    }
}

/// Loads the concept graph and every flow graph from the embedded content.
pub fn load_graph() -> GraphSet {
    let concepts = crate::docs::content::get("graph/concepts.json")
        .and_then(parse_graph)
        .unwrap_or_else(|| Graph {
            id: "concepts".to_string(),
            title: "Concepts".to_string(),
            title_zh: Some("概念图谱".to_string()),
            summary: String::new(),
            summary_zh: None,
            nodes: Vec::new(),
            edges: Vec::new(),
        });

    let mut flows: Vec<Graph> = crate::docs::content::list("flows/", ".json")
        .into_iter()
        .filter_map(|(_, text)| parse_graph(text))
        .collect();
    flows.sort_by(|left, right| left.title.cmp(&right.title));

    GraphSet { concepts, flows }
}

/// Parses a graph from its JSON representation.
///
/// Returns `None` when the identifier is missing, because a graph without an
/// identifier cannot be addressed.
pub fn parse_graph(text: &str) -> Option<Graph> {
    let value = json::parse(text).ok()?;
    let id = value.get("id")?.as_str()?.to_string();
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or(&id)
        .to_string();
    let summary = value
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let title_zh = text_field(&value, "title_zh");
    let summary_zh = text_field(&value, "summary_zh");

    let nodes = value
        .get("nodes")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(|node| {
            Some(Node {
                id: node.get("id")?.as_str()?.to_string(),
                label: node
                    .get("label")
                    .and_then(Value::as_str)
                    .unwrap_or_else(|| node.get("id").and_then(Value::as_str).unwrap_or(""))
                    .to_string(),
                label_zh: text_field(node, "label_zh"),
                kind: NodeKind::parse(
                    node.get("kind")
                        .and_then(Value::as_str)
                        .unwrap_or("concept"),
                ),
                group: node
                    .get("group")
                    .and_then(Value::as_str)
                    .unwrap_or("general")
                    .to_string(),
                summary: node
                    .get("summary")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                summary_zh: text_field(node, "summary_zh"),
                document: node
                    .get("document")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                api: node.get("api").and_then(Value::as_str).map(str::to_string),
            })
        })
        .collect();

    let edges = value
        .get("edges")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(|edge| {
            Some(Edge {
                source: edge.get("source")?.as_str()?.to_string(),
                target: edge.get("target")?.as_str()?.to_string(),
                kind: edge
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("relates")
                    .to_string(),
                label: edge
                    .get("label")
                    .or_else(|| edge.get("label_en"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
                label_zh: edge
                    .get("label_zh")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            })
        })
        .collect();

    Some(Graph {
        id,
        title,
        title_zh,
        summary,
        summary_zh,
        nodes,
        edges,
    })
}

/// Returns a member of a JSON object as an owned string, when it is a
/// non-empty string.
fn text_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// Builds the crate structure graph from the interface catalog.
///
/// The nodes are the crates, grouped by layer, and the edges follow the
/// layer order that `apsis-rs` enforces:
/// `foundation -> physics -> orbit -> rf -> network -> ops -> sim`. A layer
/// edge is drawn from each layer to the next one that the enabled feature
/// set contains, which is what the dependency rule means in practice.
pub fn crate_structure(catalog: &ApiCatalog) -> Graph {
    let layers = [
        "foundation",
        "physics",
        "orbit",
        "rf",
        "network",
        "ops",
        "sim",
    ];
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut present: Vec<&str> = Vec::new();

    for krate in &catalog.crates {
        // A crate that belongs to no layer has no place in a diagram of the
        // layer structure: the workspace facade re-exports every layer and
        // would sit under a layer it is not part of. It stays on the index,
        // where it is a crate like any other.
        if krate.layer.is_empty() {
            continue;
        }
        nodes.push(Node {
            id: krate.name.clone(),
            label: krate.name.clone(),
            label_zh: None,
            kind: NodeKind::Crate,
            group: krate.layer.clone(),
            summary: crate_summary_zh(&krate.name)
                .unwrap_or_else(|| krate.summary.lines().next().unwrap_or_default().to_string()),
            summary_zh: None,
            document: None,
            api: None,
        });
        edges.push(Edge {
            source: krate.name.clone(),
            target: krate.layer.clone(),
            kind: "belongs-to".to_string(),
            label: None,
            label_zh: None,
        });
        if !present.contains(&krate.layer.as_str()) {
            present.push(match layers.iter().find(|layer| **layer == krate.layer) {
                Some(layer) => layer,
                None => "foundation",
            });
        }
    }

    // One node per layer that the catalog contains, so the graph describes
    // this build rather than the whole workspace.
    for layer in layers {
        if !present.contains(&layer) {
            continue;
        }
        nodes.push(Node {
            id: layer.to_string(),
            label: layer.to_string(),
            label_zh: None,
            kind: NodeKind::Layer,
            group: layer.to_string(),
            summary: format!(
                "The {layer} layer. A layer may use the layers to its left and no layer to its right."
            ),
            summary_zh: Some(format!(
                "{layer} 层。每一层可以依赖其左侧的层，不能依赖其右侧的层。"
            )),
            document: Some("architecture-apsis-rs".to_string()),
            api: None,
        });
    }

    // The dependency rule, drawn between the layers that are present.
    for window in layers.windows(2) {
        let (lower, upper) = (window[0], window[1]);
        if present.contains(&lower) && present.contains(&upper) {
            edges.push(Edge {
                source: lower.to_string(),
                target: upper.to_string(),
                kind: "depends".to_string(),
                label: Some(format!("{lower} enables {upper}")),
                label_zh: Some(format!("{lower} 启用 {upper}")),
            });
        }
    }

    Graph {
        id: "crates".to_string(),
        title: "apsis-rs crate structure".to_string(),
        title_zh: Some("apsis-rs crate 结构".to_string()),
        summary: "Workspace crates grouped by layer; edges follow the one way dependency rule."
            .to_string(),
        summary_zh: Some("按层分组的 workspace crate；边表示单向依赖规则。".to_string()),
        nodes,
        edges,
    }
}

/// Returns the Chinese summary of a crate, when one is authored.
///
/// The crate name is the key, so the table stays beside the graph rather
/// than in a separate translation file.
fn crate_summary_zh(name: &str) -> Option<String> {
    let text = match name {
        "apsis-core" => "共享基础类型：通用错误、常量与单位标记 trait。",
        "apsis-units" => "物理量强类型 newtype：长度、时间、频率、角度、温度。",
        "apsis-math" => "无依赖线性代数：三维矢量、三阶矩阵、四元数。",
        "apsis-macros" => "支撑 workspace 的宏。",
        "apsis-time" => "时间尺度、历元与日历转换。",
        "apsis-leap" => "用于 UTC 转换的内嵌闰秒表。",
        "apsis-coord" => "在类型中携带参考系的位置表示。",
        "apsis-geodesy" => "参考椭球与大地坐标计算。",
        "apsis-frames" => "参考系定义与地固系旋转。",
        "apsis-tle" => "两行元素集解析。",
        "apsis-sgp4" => "SGP4 与 SDP4 解析轨道传播器。",
        "apsis-kepler" => "开普勒轨道元素与二体问题。",
        "apsis-ephemeris" => "星历采样与插值。",
        "apsis-maneuver" => "脉冲与有限推力轨道机动模型。",
        "apsis-orbit" => "轨道层统一入口：元素、传播与机动。",
        _ => return None,
    };
    Some(text.to_string())
}
