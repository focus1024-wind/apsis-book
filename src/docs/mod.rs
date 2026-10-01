//! Concept documentation and the extracted `apsis-rs` interface catalog.
//!
//! The module owns everything the service knows about documents:
//!
//! - [`content`] makes the embedded `content/` files available.
//! - [`markdown`] turns a document body into HTML.
//! - [`document`] models a document, the chapter tree, and the search index.
//! - [`api`] models the interface catalog extracted from the `apsis-rs`
//!   sources.
//!
//! [`load_documents`] and [`load_api_catalog`] assemble the two collections
//! from the embedded content at startup.
//!
//! A document is written once in English as `concepts/<id>.md`. A Chinese
//! translation is the file `concepts/<id>.zh.md`, whose metadata and body
//! replace the English ones for a reader whose language is Chinese. A
//! document without a translation is served in English to every reader.

pub mod api;
pub mod content;
pub mod document;
pub mod markdown;

#[allow(unused_imports)]
pub use api::ApiItem;
pub use api::{ApiCatalog, ApiCrate};

pub use document::{Document, DocumentVariant, Language, SearchIndex, Tree};

/// Parses every embedded concept document, merging the translations.
///
/// A file that fails to parse is skipped; the service reports the remaining
/// documents and the document route answers with a not found status for the
/// missing one.
pub fn load_documents() -> Vec<Document> {
    // The English files are the base of the collection. The identifier is
    // the file stem with the language suffix removed, so a translation and
    // its original meet on the same key.
    let mut order: Vec<String> = Vec::new();
    let mut documents: std::collections::HashMap<String, Document> =
        std::collections::HashMap::new();

    for (path, text) in content::list("concepts/", ".md") {
        let Some(stem) = path
            .strip_prefix("concepts/")
            .and_then(|name| name.strip_suffix(".md"))
        else {
            continue;
        };
        let language = Language::of_file(stem);
        if language != Language::English {
            continue;
        }
        if let Some(document) = Document::parse(stem, Language::English, text) {
            order.push(stem.to_string());
            documents.insert(stem.to_string(), document);
        }
    }

    for (path, text) in content::list("concepts/", ".zh.md") {
        let Some(stem) = path
            .strip_prefix("concepts/")
            .and_then(|name| name.strip_suffix(".zh.md"))
        else {
            continue;
        };
        // A translation without an English original is ignored rather than
        // served on its own, so that the collection always has one entry per
        // identifier.
        if !documents.contains_key(stem) {
            continue;
        }
        let Some(translation) = DocumentVariant::parse(text) else {
            continue;
        };
        let tags_zh = crate::docs::document::tags_of(text);
        if let Some(document) = documents.get_mut(stem) {
            document.chinese = Some(translation);
            document.tags_zh = tags_zh;
        }
    }

    order
        .into_iter()
        .filter_map(|id| documents.remove(&id))
        .collect()
}

/// Parses the embedded interface catalog, applying the translation when
/// one is embedded.
pub fn load_api_catalog() -> ApiCatalog {
    let Some(text) = content::get("api.json") else {
        return ApiCatalog {
            crates: Vec::new(),
            generated_from: Vec::new(),
            item_names: std::collections::HashSet::new(),
        };
    };
    let translation = content::get("api.zh.json").and_then(|text| crate::json::parse(text).ok());
    api::parse_with(text, translation.as_ref())
}
