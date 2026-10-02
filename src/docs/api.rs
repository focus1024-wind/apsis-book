//! The interface catalog extracted from the `apsis-rs` sources.
//!
//! Task two of the service is to present the public interface of `apsis-rs`
//! as it is written in the source, rather than as someone retyped it. The
//! extraction is performed by `scripts/extract_api.sh`, which reads the
//! source tree, collects the module documentation and the public items with
//! their doc comments, and writes `content/api.json`. This module parses that
//! file and exposes it to the HTTP layer.
//!
//! The generator is developed in tandem with the project so that the
//! catalog matches the source it was made from; when an item is added or
//! its signature changes, the script is rerun. It does not read the source
//! at run time, so the binary does not depend on a checkout of `apsis-rs`.

use crate::json::{self, Value};

/// A single public item of a crate.
#[derive(Debug, Clone)]
pub struct ApiItem {
    /// Kind of item: `struct`, `enum`, `fn`, `trait`, `const`, or `type`.
    pub kind: String,
    /// Item name.
    pub name: String,
    /// Signature as written in the source, on one line.
    pub signature: String,
    /// Documentation comment, with the leading `///` markers removed.
    pub doc: String,
    /// The documentation comment in Chinese, when a translation exists.
    ///
    /// The author writes the doc comment in English and then in Chinese, and
    /// the extractor splits the two halves, so both languages come from the
    /// one comment beside the code.
    pub doc_zh: Option<String>,
    /// `true` when the item is an associated item rather than a module
    /// level one.
    pub associated: bool,
}

impl ApiItem {
    /// Returns `true` when the item is a function.
    ///
    /// Used where a caller distinguishes the callable items, such as the
    /// item listing of a crate.
    #[allow(dead_code)]
    pub fn is_function(&self) -> bool {
        self.kind == "fn"
    }
}

/// A public crate of the `apsis-rs` workspace.
#[derive(Debug, Clone)]
pub struct ApiCrate {
    /// Crate name, such as `apsis-sgp4`.
    pub name: String,
    /// Layer the crate belongs to: `foundation`, `physics`, or `orbit`.
    pub layer: String,
    /// Crate level documentation, with the leading `//!` markers removed.
    pub summary: String,
    /// The crate level documentation in Chinese, when a translation exists.
    pub summary_zh: Option<String>,
    /// Public items, in source order.
    pub items: Vec<ApiItem>,
}

impl ApiCrate {
    /// Returns the items of the given kind.
    ///
    /// The kind is the one written in the source, such as `struct` or
    /// `fn`.
    #[allow(dead_code)]
    pub fn items_of_kind(&self, kind: &str) -> Vec<&ApiItem> {
        self.items.iter().filter(|item| item.kind == kind).collect()
    }
}

/// The whole catalog.
#[derive(Debug, Clone)]
pub struct ApiCatalog {
    /// The names of every item, for resolving a doc comment link.
    pub item_names: std::collections::HashSet<String>,
    /// The crates, in dependency order.
    pub crates: Vec<ApiCrate>,
    /// Source paths the catalog was extracted from, for provenance.
    pub generated_from: Vec<String>,
}

impl ApiCatalog {
    /// Returns a crate by name.
    pub fn krate(&self, name: &str) -> Option<&ApiCrate> {
        self.crates.iter().find(|krate| krate.name == name)
    }

    /// Returns every item with the given name.
    ///
    /// A name such as `propagate_minutes` is unique in the workspace, so a
    /// search by name finds the interface a tool or an internal link refers
    /// to.
    pub fn find_item(&self, name: &str) -> Option<(&ApiCrate, &ApiItem)> {
        self.crates.iter().find_map(|krate| {
            krate
                .items
                .iter()
                .find(|item| item.name == name)
                .map(|item| (krate, item))
        })
    }

    /// Returns the number of items in the catalog.
    pub fn item_count(&self) -> usize {
        self.crates.iter().map(|krate| krate.items.len()).sum()
    }

    /// Returns `true` when the catalog is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.crates.is_empty()
    }
}

/// Parses the catalog from its JSON representation.
///
/// A malformed member is skipped rather than failing the whole document, so
/// that a partially generated catalog still serves the items it contains.
#[allow(dead_code)]
/// Parses the catalogue.
///
/// The catalogue carries both languages, because `apsis-rs` writes its doc
/// comments in English and then in Chinese. `scripts/extract_api.sh` splits
/// the two halves while it reads the sources, so no separate translation
/// file is needed and a translation cannot fall behind the source it
/// describes.
pub fn parse(text: &str) -> ApiCatalog {
    let Ok(value) = json::parse(text) else {
        return ApiCatalog {
            crates: Vec::new(),
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
    let crates: Vec<ApiCrate> = value
        .get("crates")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(parse_crate)
        .collect();
    let item_names = crates
        .iter()
        .flat_map(|krate| krate.items.iter().map(|item| item.name.clone()))
        .collect();
    ApiCatalog {
        crates,
        generated_from,
        item_names,
    }
}

/// Returns a string member of an object, when it is present and not empty.
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

/// Parses one crate.
fn parse_crate(value: &Value) -> Option<ApiCrate> {
    let name = value.get("name")?.as_str()?.to_string();
    // A crate that belongs to no layer, which is the facade that re-exports
    // every layer, is left empty. Defaulting it to a layer would file it
    // under a subject it does not belong to.
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
    let summary_zh = string_member(value, "summary_zh");
    Some(ApiCrate {
        name,
        layer,
        summary,
        summary_zh,
        items,
    })
}

/// Parses one item.
pub(crate) fn parse_item(value: &Value, _krate: &str, _owner: Option<&str>) -> Option<ApiItem> {
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
