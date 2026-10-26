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
//!
//! 服务提供两种图：
//!
//! - 概念图谱，撰写于 `content/graph/concepts.json`。其节点是概念，边是它们之间
//!   的关系，例如 `uses`、`refines` 或 `contrasts`。节点携带解释它的文档的
//!   标识，因此前端可以在节点被点击时打开那篇文档。
//! - 流程图，每个文件一张，位于 `content/flows/`。一张流程图以带形状的节点与
//!   带分支的带标签边，描述某个算法的步骤，例如 SGP4 的传播序列。
//!
//! 两者都是同一形状的纯 JSON，因此前端一个渲染器即可处理任一者。布局在前端由
//! 节点种类算出；后端只提供结构，这让撰写的数据与任何特定的渲染方式无关。

use crate::docs::ApiCatalog;
use crate::json::{self, Value};

/// The kind of a graph node.
///
/// The kind selects how a node is drawn. The set is intentionally small so
/// that a new graph can be authored without touching the renderer.
///
/// 图节点的种类。
///
/// 种类决定节点如何绘制。该集合刻意保持很小，使新增一张图无需改动渲染器。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// A concept in the knowledge graph.
    ///
    /// 知识图谱中的一个概念。
    Concept,
    /// A module of the `apsis` crate.
    ///
    /// `apsis` crate 中的一个模块。
    Module,
    /// A layer of the `apsis` crate.
    ///
    /// `apsis` crate 中的一个层。
    Layer,
    /// A step of an algorithm.
    ///
    /// 算法的一个步骤。
    Step,
    /// A decision in an algorithm, with more than one outgoing edge.
    ///
    /// 算法中的一个判定，有多于一条出边。
    Decision,
    /// A terminal step, such as a returned value or an error.
    ///
    /// 一个终结步骤，例如返回的值或一个错误。
    Terminal,
    /// A data item, such as an element set or an epoch.
    ///
    /// 一个数据项，例如元素集或历元。
    Data,
}

impl NodeKind {
    /// Parses a kind name, defaulting to [`NodeKind::Concept`].
    ///
    /// 解析一个种类名，默认为 [`NodeKind::Concept`]。
    pub fn parse(name: &str) -> Self {
        match name {
            "module" => Self::Module,
            "layer" => Self::Layer,
            "step" => Self::Step,
            "decision" => Self::Decision,
            "terminal" => Self::Terminal,
            "data" => Self::Data,
            _ => Self::Concept,
        }
    }

    /// Returns the kind name as it is written in the data.
    ///
    /// 以数据中书写的原样返回种类名。
    pub const fn name(self) -> &'static str {
        match self {
            Self::Concept => "concept",
            Self::Module => "module",
            Self::Layer => "layer",
            Self::Step => "step",
            Self::Decision => "decision",
            Self::Terminal => "terminal",
            Self::Data => "data",
        }
    }
}

/// A node of a graph.
///
/// 图的一个节点。
#[derive(Debug, Clone)]
pub struct Node {
    /// Identifier, unique within the graph.
    ///
    /// 标识，在图内唯一。
    pub id: String,
    /// Display label in English.
    ///
    /// 英文显示标签。
    pub label: String,
    /// Display label in Chinese, or `None` when the label is a name that is
    /// written the same way in both languages.
    ///
    /// 中文显示标签；当标签是一个在两种语言中写法相同的名字时为 `None`。
    pub label_zh: Option<String>,
    /// Kind, selecting how the node is drawn.
    ///
    /// 种类，决定节点如何绘制。
    pub kind: NodeKind,
    /// Chapter or group the node belongs to, used for colouring.
    ///
    /// 节点所属的章或分组，用于着色。
    pub group: String,
    /// One sentence description in English.
    ///
    /// 一句话英文描述。
    ///
    /// 一句话英文描述。
    pub summary: String,
    /// One sentence description in Chinese, when one is authored.
    ///
    /// 一句话中文描述，若已撰写。
    ///
    /// 一句话中文描述，若已撰写。
    pub summary_zh: Option<String>,
    /// Identifier of the document that explains the node, when one exists.
    ///
    /// 解释该节点的文档标识，若存在。
    pub document: Option<String>,
    /// Identifier of the interface item, when the node names one.
    ///
    /// 接口条目标识，若该节点指名了某个条目。
    pub api: Option<String>,
}

/// An edge of a graph.
///
/// 图的一条边。
#[derive(Debug, Clone)]
pub struct Edge {
    /// Source node identifier.
    ///
    /// 起点节点标识。
    pub source: String,
    /// Target node identifier.
    ///
    /// 终点节点标识。
    pub target: String,
    /// Relationship name, such as `uses`.
    ///
    /// 关系名，例如 `uses`。
    pub kind: String,
    /// Optional label drawn on the edge, in English.
    ///
    /// 绘制在边上的可选标签，英文。
    pub label: Option<String>,
    /// The same label in Chinese, when one is authored.
    ///
    /// 同一标签的中文，若已撰写。
    pub label_zh: Option<String>,
}

/// A graph, either the concept graph or one flow graph.
///
/// 一张图，或为概念图谱，或为一张流程图。
#[derive(Debug, Clone)]
pub struct Graph {
    /// Identifier of the graph.
    ///
    /// 图的标识。
    pub id: String,
    /// Display title in English.
    ///
    /// 英文显示标题。
    pub title: String,
    /// Display title in Chinese, when one is authored.
    ///
    /// 中文显示标题，若已撰写。
    pub title_zh: Option<String>,
    /// One sentence description in English.
    pub summary: String,
    /// One sentence description in Chinese, when one is authored.
    pub summary_zh: Option<String>,
    /// Nodes.
    ///
    /// 各节点。
    pub nodes: Vec<Node>,
    /// Edges.
    ///
    /// 各边。
    pub edges: Vec<Edge>,
}

impl Graph {
    /// Returns a node by identifier.
    ///
    /// Used to resolve the endpoints of an edge, which is what the check
    /// for a dangling edge does.
    ///
    /// 按标识返回一个节点。
    ///
    /// 用于解析一条边的两个端点，悬空边的检查正是这么做的。
    #[allow(dead_code)]
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// Returns the edges that leave a node.
    ///
    /// 返回离开某个节点的各条边。
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
///
/// 从内嵌内容载入概念图谱与每一张流程图。
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
///
/// 从 JSON 表示解析一张图。
///
/// 标识缺失时返回 `None`，因为没有标识的图无法被寻址。
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
///
/// 把 JSON 对象的某个成员返回为自有字符串，当它是非空字符串时。
fn text_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// Builds the module structure graph from the interface catalog.
///
/// The nodes are the modules of the `apsis` crate, grouped by layer, and the
/// edges follow the layer order the crate enforces:
/// `foundation -> physics -> orbit -> rf -> network -> ops -> sim`. A layer
/// edge is drawn from each layer to the next one the catalog contains, which
/// is what the dependency rule means in practice.
///
/// 从接口目录构建模块结构图。
///
/// 节点是 `apsis` crate 的各模块，按层分组；边沿该 crate 强制的层序：
/// `foundation -> physics -> orbit -> rf -> network -> ops -> sim`。
/// 层边从每一层画向目录中包含的下一层，这正是依赖规则在实践中
/// 的含义。
pub fn module_structure(catalog: &ApiCatalog) -> Graph {
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

    for module in &catalog.modules {
        // A module that belongs to no layer has no place in a diagram of the
        // layer structure. It stays on the index, where it is a module like
        // any other.

        // 不属于任何层的模块在层结构图中没有位置。它留在索引里，在那里它与别的
        // 模块无异。
        if module.layer.is_empty() {
            continue;
        }
        nodes.push(Node {
            id: module.name.clone(),
            label: module.name.clone(),
            label_zh: None,
            kind: NodeKind::Module,
            group: module.layer.clone(),
            summary: opening_line(&module.summary),
            summary_zh: module_summary_zh(&module.name),
            document: None,
            api: Some(module.name.clone()),
        });
        edges.push(Edge {
            source: module.name.clone(),
            target: module.layer.clone(),
            kind: "belongs-to".to_string(),
            label: None,
            label_zh: None,
        });
        if !present.contains(&module.layer.as_str()) {
            present.push(match layers.iter().find(|layer| **layer == module.layer) {
                Some(layer) => layer,
                None => "foundation",
            });
        }
    }

    // One node per layer that the catalog contains, so the graph describes
    // this build rather than the whole crate.

    // 目录中包含的每一层一个节点，因此该图描述的是本次构建，而不是整个 crate。
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

    // 依赖规则，画在存在的各层之间。
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
        id: "modules".to_string(),
        title: "apsis module structure".to_string(),
        title_zh: Some("apsis 模块结构".to_string()),
        summary: "The modules of the apsis crate grouped by layer; edges follow the one way dependency rule."
            .to_string(),
        summary_zh: Some("按层分组的 apsis 模块；边表示单向依赖规则。".to_string()),
        nodes,
        edges,
    }
}

/// Returns the opening paragraph of a module summary, for a graph node.
///
/// The module documentation opens with a sentence or two that say what the
/// module is, which is what a node has room for. The paragraph is joined onto
/// one line because the source wraps it to a column and a node does not.
///
/// 返回模块概要的首段，供图节点使用。
///
/// 模块文档以一两句话开头，说明该模块是什么，这正是节点所能容纳的内容。该段被
/// 接成一行，因为源码按列宽折行，而节点不折行。
fn opening_line(summary: &str) -> String {
    let mut lines = Vec::new();
    for line in summary.lines() {
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        lines.push(line);
    }
    lines.join(" ")
}

/// Returns the Chinese summary of a module, when one is authored.
///
/// The module name is the key, so the table stays beside the graph rather
/// than in a separate translation file. A module added to the crate without
/// an entry falls back to its English opening line, which is why the table
/// covers every module the crate has today.
///
/// 返回某个模块的中文摘要，若已撰写。
///
/// 以模块名为键，因此该表与图放在一起，而不在单独的译文文件中。新增而未在此
/// 登记的模块回退到其英文首句，故该表覆盖该 crate 今天拥有的每一个模块。
fn module_summary_zh(name: &str) -> Option<String> {
    let text = match name {
        "apsis_core" => "共享基础类型：通用错误、常量与单位标记 trait。",
        "apsis_macros" => "共享宏的保留路径，当前不含任何宏。",
        "apsis_math" => "无依赖线性代数：三维矢量、三阶矩阵、四元数。",
        "apsis_units" => "物理量强类型 newtype：长度、时间、频率、角度、温度。",
        "apsis_coord" => "在类型中携带参考系的位置表示。",
        "apsis_frames" => "参考系定义与惯性系到地固系的旋转。",
        "apsis_geodesy" => "参考椭球与大地坐标计算。",
        "apsis_leap" => "用于 UTC 转换的内嵌闰秒表。",
        "apsis_time" => "时间尺度、历元与日历转换。",
        "apsis_ephemeris" => "星历采样与插值。",
        "apsis_kepler" => "开普勒轨道元素与二体问题。",
        "apsis_maneuver" => "脉冲与有限推力轨道机动模型。",
        "apsis_sgp4" => "SGP4 与 SDP4 解析轨道传播器。",
        "apsis_tle" => "两行元素集解析。",
        "apsis_orbit" => "轨道层统一入口：元素、传播与机动。",
        "apsis_antenna" => "天线方向图与增益模型。",
        "apsis_atmosphere" => "大气衰减与折射模型。",
        "apsis_doppler" => "多普勒频移计算。",
        "apsis_interference" => "干扰与载干比计算。",
        "apsis_link" => "链路预算计算。",
        "apsis_noise" => "噪声温度与噪声系数模型。",
        "apsis_rf" => "射频层统一入口：分贝、自由空间损耗、功率与增益。",
        "apsis_access" => "地面站可见窗口与访问计划。",
        "apsis_constellation" => "星座定义与实例化。",
        "apsis_coverage" => "地面覆盖分析：足迹几何与可见性。",
        "apsis_handover" => "移动终端的切换决策。",
        "apsis_isl" => "星间链路几何与可见性。",
        "apsis_mesh" => "网状连通性抽象。",
        "apsis_routing" => "时变拓扑上的路由。",
        "apsis_topology" => "时变网络拓扑。",
        "apsis_conflict" => "资源与时间冲突检测。",
        "apsis_resource" => "卫星运营的资源建模。",
        "apsis_schedule" => "调度结果的表示与评估。",
        "apsis_scheduler" => "卫星任务规划的调度算法。",
        "apsis_task" => "卫星运营的任务建模。",
        "apsis_event" => "仿真事件类型与事件队列。",
        "apsis_output" => "仿真输出记录。",
        "apsis_scenario" => "场景定义与载入。",
        "apsis_sim_engine" => "离散事件仿真引擎。",
        "apsis_sim" => "仿真层统一入口：场景、事件、引擎与输出。",
        _ => return None,
    };
    Some(text.to_string())
}
