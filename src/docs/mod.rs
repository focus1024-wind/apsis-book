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

pub mod api;
pub mod content;
pub mod document;
pub mod markdown;

#[allow(unused_imports)]
pub use api::ApiItem;
pub use api::{ApiCatalog, ApiCrate};

pub use document::{Document, SearchIndex, Tree};

/// Parses every embedded concept document.
///
/// A file that fails to parse is skipped; the service reports the remaining
/// documents and the document route answers with a not found status for the
/// missing one.
pub fn load_documents() -> Vec<Document> {
    content::list("concepts/", ".md")
        .into_iter()
        .filter_map(|(_, text)| Document::parse(text))
        .collect()
}

/// Parses the embedded interface catalog.
pub fn load_api_catalog() -> ApiCatalog {
    match content::get("api.json") {
        Some(text) => api::parse(text),
        None => ApiCatalog {
            crates: Vec::new(),
            generated_from: Vec::new(),
        },
    }
}
