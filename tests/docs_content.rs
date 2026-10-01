//! Tests for the embedded content table.
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

use apsis_book::docs::content::*;

#[test]
fn concept_documents_are_embedded() {
    let documents = list("concepts/", ".md");
    assert!(documents.len() >= 10, "expected a populated concept set");
}

#[test]
fn graph_data_is_embedded() {
    assert!(get("graph/concepts.json").is_some());
}
