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
pub fn parse(text: &str) -> ApiCatalog {
    let Ok(value) = json::parse(text) else {
        return ApiCatalog {
            crates: Vec::new(),
            generated_from: Vec::new(),
        };
    };
    let generated_from = value
        .get("generated_from")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(|path| path.as_str().map(str::to_string))
        .collect();
    let crates = value
        .get("crates")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(parse_crate)
        .collect();
    ApiCatalog {
        crates,
        generated_from,
    }
}

/// Parses one crate.
fn parse_crate(value: &Value) -> Option<ApiCrate> {
    let name = value.get("name")?.as_str()?.to_string();
    let layer = value
        .get("layer")
        .and_then(Value::as_str)
        .unwrap_or("foundation")
        .to_string();
    let summary = value
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let items = value
        .get("items")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(parse_item)
        .collect();
    Some(ApiCrate {
        name,
        layer,
        summary,
        items,
    })
}

/// Parses one item.
fn parse_item(value: &Value) -> Option<ApiItem> {
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
    Some(ApiItem {
        kind,
        name,
        signature,
        doc,
        associated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "generated_from": ["crates/orbit/apsis-sgp4/src/lib.rs"],
      "crates": [
        {
          "name": "apsis-sgp4",
          "layer": "orbit",
          "summary": "The SGP4 and SDP4 analytical orbit propagator.",
          "items": [
            {"kind": "struct", "name": "Satellite", "signature": "pub struct Satellite { /* private */ }", "doc": "A prepared satellite.", "associated": false},
            {"kind": "fn", "name": "propagate_minutes", "signature": "pub fn propagate_minutes(&self, minutes: f64) -> Result<State, PropagationError>", "doc": "Propagates by minutes.", "associated": true}
          ]
        }
      ]
    }"#;

    #[test]
    fn parses_the_catalog() {
        let catalog = parse(SAMPLE);
        assert_eq!(catalog.crates.len(), 1);
        assert_eq!(catalog.item_count(), 2);
        let krate = catalog.krate("apsis-sgp4").unwrap();
        assert_eq!(krate.layer, "orbit");
        assert_eq!(krate.items_of_kind("fn").len(), 1);
        let (_, item) = catalog.find_item("propagate_minutes").unwrap();
        assert!(item.is_function());
        assert!(item.associated);
    }

    #[test]
    fn malformed_input_yields_an_empty_catalog() {
        assert!(parse("not json").is_empty());
        assert!(parse("{}").is_empty());
    }
}
