//! The seams the tests use.
//!
//! The service tests live in `tests/` beside this file rather than inside
//! the modules, so that `src/` holds no test code. An integration test is a
//! separate crate and can reach only what the library makes public, which
//! would mean making an internal helper public to test one line of it.
//!
//! The items below stay `pub(crate)` where they are defined, and are
//! reached through a wrapper here. A `pub(crate)` item cannot be re-exported
//! as `pub`, because Rust does not let visibility be widened twice, so a
//! wrapper is the only way to offer a private thing outward without making
//! it public where it lives.
//!
//! Everything here is an implementation detail of the module named, and none
//! of it is covered by the service's stability promise. The wrapper is what
//! a test calls; the original is free to change shape behind it, as long as
//! the wrapper is kept in step. When a helper is no longer worth testing
//! directly, delete it from here rather than promote it.

use crate::docs::api::ApiItem;
use crate::http::request::RequestError;
use crate::http::response::Response;
use crate::json::Value;

/// The cache policy a file without a content hash is served under.
pub const UNHASHED_CACHE_POLICY: &str = crate::api::UNHASHED_CACHE_POLICY;

/// Returns the cache policy for a path.
pub fn cache_policy(path: &str) -> &'static str {
    crate::api::cache_policy(path)
}

/// Returns `true` when a path carries a content hash in its name.
pub fn is_content_hashed(path: &str) -> bool {
    crate::api::is_content_hashed(path)
}

/// Returns the opening sentence of a summary, for a card that links onward.
pub fn crate_summary_brief(summary: &str) -> String {
    crate::api::crate_summary_brief(summary)
}

/// Parses one interface item of the catalogue.
pub fn parse_item(value: &Value, krate: &str, owner: Option<&str>) -> Option<ApiItem> {
    crate::docs::api::parse_item(value, krate, owner)
}

/// Splits text into the tokens the search index is built from.
pub fn tokenise(text: &str) -> Vec<String> {
    crate::docs::document::tokenise(text)
}

/// Returns the response a request error becomes.
pub fn error_response(error: RequestError) -> Response {
    crate::http::server::error_response(error)
}

/// Returns the MathML of a formula, without the wrapper that carries it.
///
/// The two steps are one call here because the token type between them is
/// private to the module that owns it and is of no use to a test.
pub fn mathml(latex: &str) -> String {
    crate::docs::mathml::to_mathml(&crate::docs::mathml::normalize(latex))
}
/// Builds a JSON object literal.
///
/// The tests read better written this way than spelled out, and the macro is an
/// internal construction helper, so it is named here with the other seams.
pub use crate::json_object;
