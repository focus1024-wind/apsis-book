//! Access to the authored text content embedded in the binary.
//!
//! `build.rs` scans the `content/` directory and generates
//! `$OUT_DIR/embedded_content.rs`, whose `CONTENT` table maps the path
//! relative to `content/` to the file text. The document layer consumes that
//! table, so the service never reads a content file from the filesystem at
//! run time.
//!
//! The interface catalog under `content/api.json` is generated from the
//! `apsis-rs` source by `scripts/extract_api.sh` and travels through the
//! same table.

include!(concat!(env!("OUT_DIR"), "/embedded_content.rs"));

/// Returns the embedded text for a path relative to `content/`.
pub fn get(path: &str) -> Option<&'static str> {
    CONTENT
        .iter()
        .find(|(name, _)| *name == path)
        .map(|(_, text)| *text)
}

/// Returns every embedded entry whose path starts with `prefix` and ends
/// with `suffix`.
pub fn list(prefix: &str, suffix: &str) -> Vec<(&'static str, &'static str)> {
    CONTENT
        .iter()
        .filter(|(name, _)| name.starts_with(prefix) && name.ends_with(suffix))
        .map(|(name, text)| (*name, *text))
        .collect()
}

/// Returns `true` when at least one content file is embedded.
///
/// The service reports this in the health response, so a build without
/// content says so rather than serving an empty documentation set.
#[allow(dead_code)]
pub fn is_populated() -> bool {
    !CONTENT.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concept_documents_are_embedded() {
        let documents = list("concepts/", ".md");
        assert!(documents.len() >= 10, "expected a populated concept set");
    }

    #[test]
    fn graph_data_is_embedded() {
        assert!(get("graph/concepts.json").is_some());
    }
}
