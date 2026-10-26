//! The interface catalog extracted from the `apsis` sources.
//!
//! Task two of the service is to present the public interface of the `apsis`
//! crate as it is written in the source, rather than as someone retyped it.
//! The extraction is performed by `scripts/extract_api.sh`, which reads the
//! module tree, collects the module documentation and the public items with
//! their doc comments, and writes `content/api.json`. This module parses that
//! file and exposes it to the HTTP layer.
//!
//! The generator is developed in tandem with the project so that the
//! catalog matches the source it was made from; when an item is added or
//! its signature changes, the script is rerun. It does not read the source
//! at run time, so the binary does not depend on a checkout of `apsis-rs`.
//!
//! 从 `apsis` 源码提取的接口目录。
//!
//! 服务的第二项任务，是原样呈现 `apsis` crate 在源码中写下的公共接口，而不是别人
//! 重打一遍的版本。提取由 `scripts/extract_api.sh` 完成：它读取模块树，收集模块
//! 文档与各公共条目的文档注释，写出 `content/api.json`。本模块解析该文件并把它
//! 暴露给 HTTP 层。
//!
//! 生成器与项目同步演进，以使目录与它所来自的源码一致；新增条目或签名变化时重跑
//! 脚本。它不在运行期读取源码，因此二进制不依赖 `apsis-rs` 的检出。

use crate::json::{self, Value};

/// A single public item of a module.
///
/// 某个模块的一个公共条目。
#[derive(Debug, Clone)]
pub struct ApiItem {
    /// Kind of item: `struct`, `enum`, `fn`, `trait`, `const`, or `type`.
    ///
    /// 条目的种类：`struct`、`enum`、`fn`、`trait`、`const` 或 `type`。
    pub kind: String,
    /// Item name.
    /// 条目名。
    pub name: String,
    /// Signature as written in the source, on one line.
    /// 源码中书写的签名，排在同一行内。
    pub signature: String,
    /// Documentation comment, with the leading `///` markers removed.
    /// 文档注释，已去掉各行开头的 `///` 标记。
    pub doc: String,
    /// The documentation comment in Chinese, when a translation exists.
    ///
    /// The author writes the doc comment in English and then in Chinese, and
    /// the extractor splits the two halves, so both languages come from the
    /// one comment beside the code.
    ///
    /// 该文档注释的中文部分，若有译文。
    ///
    /// 作者把文档注释先写成英文、再写成中文，提取器把两半拆开，因此两种语言都
    /// 出自代码旁的同一段注释。
    pub doc_zh: Option<String>,
    /// `true` when the item is an associated item rather than a module
    /// level one.
    ///
    /// 当该条目是关联条目而非模块级条目时为 `true`。
    pub associated: bool,
}

impl ApiItem {
    /// Returns `true` when the item is a function.
    ///
    /// Used where a caller distinguishes the callable items, such as the
    /// item listing of a module.
    ///
    /// 当该条目是函数时返回 `true`。
    ///
    /// 用于调用方需要区分可调用条目的地方，例如某个模块的条目列表。
    #[allow(dead_code)]
    pub fn is_function(&self) -> bool {
        self.kind == "fn"
    }
}

/// A public module of the `apsis` crate.
///
/// The module name is the path the crate re-exports it under, so
/// `apsis_sgp4` is reached as `apsis::apsis_sgp4`. The name is kept whole
/// rather than split into a subject, because that is the path a reader types
/// and the name a document link refers to.
///
/// `apsis` crate 的一个公共模块。
///
/// 模块名即 crate 重导出它的路径，因此 `apsis_sgp4` 以 `apsis::apsis_sgp4`
/// 到达。名字整体保留而不拆成主题，因为读者键入的就是这条路径，文档链接所指的
/// 也是这个名字。
#[derive(Debug, Clone)]
pub struct ApiModule {
    /// Module name, such as `apsis_sgp4`.
    /// 模块名，例如 `apsis_sgp4`。
    pub name: String,
    /// Layer the module belongs to: `foundation`, `physics`, `orbit`, and so
    /// on.
    /// 该模块所属的层：`foundation`、`physics`、`orbit` 等。
    pub layer: String,
    /// Module level documentation, with the leading `//!` markers removed.
    /// 模块级文档，已去掉各行开头的 `//!` 标记。
    pub summary: String,
    /// The module level documentation in Chinese, when a translation exists.
    /// 模块级文档的中文部分，若有译文。
    pub summary_zh: Option<String>,
    /// Public items, in source order.
    /// 公共条目，按源码顺序。
    pub items: Vec<ApiItem>,
}

impl ApiModule {
    /// Returns the items of the given kind.
    ///
    /// The kind is the one written in the source, such as `struct` or
    /// `fn`.
    ///
    /// 返回给定种类的条目。
    ///
    /// 种类即源码中书写的那一个，例如 `struct` 或 `fn`。
    #[allow(dead_code)]
    pub fn items_of_kind(&self, kind: &str) -> Vec<&ApiItem> {
        self.items.iter().filter(|item| item.kind == kind).collect()
    }
}

/// The whole catalog.
///
/// 整个目录。
#[derive(Debug, Clone)]
pub struct ApiCatalog {
    /// The names of every item, for resolving a doc comment link.
    /// 每个条目的名字，用于解析文档注释中的链接。
    pub item_names: std::collections::HashSet<String>,
    /// The modules, in dependency order.
    /// 各模块，按依赖顺序。
    pub modules: Vec<ApiModule>,
    /// Source paths the catalog was extracted from, for provenance.
    /// 目录所来自的源码路径，用于溯源自何处。
    pub generated_from: Vec<String>,
}

impl ApiCatalog {
    /// Returns a module by name.
    /// 按名字返回一个模块。
    pub fn module(&self, name: &str) -> Option<&ApiModule> {
        self.modules.iter().find(|module| module.name == name)
    }

    /// Returns every item with the given name.
    ///
    /// A name such as `propagate_minutes` is unique in the crate, so a
    /// search by name finds the interface a tool or an internal link refers
    /// to.
    ///
    /// 返回具有给定名字的每一个条目。
    ///
    /// 诸如 `propagate_minutes` 这样的名字在该 crate 内是唯一的，因此按名字查找
    /// 就能找到某个工具或内部链接所指的接口。
    pub fn find_item(&self, name: &str) -> Option<(&ApiModule, &ApiItem)> {
        self.modules.iter().find_map(|module| {
            module
                .items
                .iter()
                .find(|item| item.name == name)
                .map(|item| (module, item))
        })
    }

    /// Returns the number of items in the catalog.
    /// 返回目录中的条目数。
    pub fn item_count(&self) -> usize {
        self.modules.iter().map(|module| module.items.len()).sum()
    }

    /// Returns `true` when the catalog is empty.
    /// 当目录为空时返回 `true`。
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}

#[allow(dead_code)]
/// Parses the catalogue.
///
/// The catalogue carries both languages, because `apsis` writes its doc
/// comments in English and then in Chinese. `scripts/extract_api.sh` splits
/// the two halves while it reads the sources, so no separate translation
/// file is needed and a translation cannot fall behind the source it
/// describes.
///
/// A malformed member is skipped rather than failing the whole document, so
/// that a partially generated catalog still serves the items it contains.
///
/// 解析接口目录。
///
/// 目录同时携带两种语言，因为 `apsis` 把文档注释先写成英文、再写成中文。
/// `scripts/extract_api.sh` 在读取源码时把两半拆开，因此无需单独的译文文件，
/// 译文也不会落后于它所描述的源码。
///
/// 畸形的成员被跳过，而不是让整份文档失败，使生成不完整的目录仍能提供它所含的
/// 条目。
pub fn parse(text: &str) -> ApiCatalog {
    let Ok(value) = json::parse(text) else {
        return ApiCatalog {
            modules: Vec::new(),
            generated_from: Vec::new(),
            item_names: std::collections::HashSet::new(),
        };
    };
    let generated_from = value
        .get("generated_from")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(|path| path.as_str().map(str::to_string))
        .collect();
    let modules: Vec<ApiModule> = value
        .get("modules")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(parse_module)
        .collect();
    let item_names = modules
        .iter()
        .flat_map(|module| module.items.iter().map(|item| item.name.clone()))
        .collect();
    ApiCatalog {
        modules,
        generated_from,
        item_names,
    }
}

/// Returns a string member of an object, when it is present and not empty.
///
/// 返回某个对象的一个字符串成员，若它存在且非空。
fn string_member(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// Returns the name of the type an item is associated with.
///
/// The catalog lists an item in source order, so the type an associated
/// function belongs to is the last type declaration before it.
///
/// 返回某个条目所关联的类型名。
///
/// 目录按源码顺序列出条目，因此一个关联函数所属的类型就是它之前最后一次类型声明。
fn owner_of(items: &[Value], index: usize) -> Option<String> {
    let mut owner = None;
    for item in items.iter().take(index + 1) {
        if let Some(kind) = item.get("kind").and_then(Value::as_str) {
            if matches!(kind, "struct" | "enum" | "trait" | "type") {
                owner = item.get("name").and_then(Value::as_str).map(str::to_string);
            }
        }
    }
    owner
}

/// Parses one module.
/// 解析一个模块。
fn parse_module(value: &Value) -> Option<ApiModule> {
    let name = value.get("name")?.as_str()?.to_string();
    // A module that belongs to no layer has no place in the layer index, so
    // its layer is left empty rather than defaulted to one it does not
    // belong to.
    //
    // 不属于任何层的模块在层索引中没有位置，故其层留空，而不是默认归到一个它并
    // 不属于的层。
    let layer = value
        .get("layer")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let summary = value
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let raw_items = value.get("items").map(Value::as_array).unwrap_or_default();
    let items = raw_items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| parse_item(item, &name, owner_of(raw_items, index).as_deref()))
        .collect();
    // The module documentation is written in English and then in Chinese;
    // the extractor separated the two.

    // 模块文档先写成英文、再写成中文；提取器已把两者分开。
    let summary_zh = string_member(value, "summary_zh");
    Some(ApiModule {
        name,
        layer,
        summary,
        summary_zh,
        items,
    })
}

/// Parses one item.
/// 解析一个条目。
pub(crate) fn parse_item(value: &Value, _module: &str, _owner: Option<&str>) -> Option<ApiItem> {
    let name = value.get("name")?.as_str()?.to_string();
    let kind = value.get("kind")?.as_str()?.to_string();
    let signature = value
        .get("signature")
        .and_then(Value::as_str)
        .unwrap_or(&name)
        .to_string();
    let doc = value
        .get("doc")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let associated = value
        .get("associated")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let doc_zh = string_member(value, "doc_zh");
    Some(ApiItem {
        kind,
        name,
        signature,
        doc,
        doc_zh,
        associated,
    })
}
