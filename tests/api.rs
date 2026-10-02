//! Tests for the router and the payloads it encodes.
//! 路由器及其编码的载荷的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.
//!
//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。
use std::sync::Arc;

use apsis_book::api::*;
use apsis_book::http::request::{Method, Request};
use apsis_book::http::response::{Response, Status};
use apsis_book::http::server::Handler;
use apsis_book::json::{self, Value};
use apsis_book::state::AppState;
use apsis_book::testkit::{
    UNHASHED_CACHE_POLICY, cache_policy, crate_summary_brief, is_content_hashed,
};

use std::collections::HashMap;

fn router() -> Router {
    Router::new(Arc::new(AppState::new()))
}

fn get(router: &Router, path: &str) -> Response {
    // The request parser splits the target, so a test that wants a query
    // supplies it in the path and it is split the same way here.

    // 请求解析器会拆分目标，因此想要查询的测试把它写在路径里，这里也以同样的
    // 方式拆分。
    let (path, query) = match path.split_once('?') {
        Some((path, query)) => (path.to_string(), Some(query.to_string())),
        None => (path.to_string(), None),
    };
    let request = Request {
        method: Method::Get,
        path,
        query,
        headers: HashMap::new(),
        body: Vec::new(),
    };
    router.handle(&request, "127.0.0.1:1".parse().unwrap())
}

fn post(router: &Router, path: &str, body: &str) -> Response {
    let request = Request {
        method: Method::Post,
        path: path.to_string(),
        query: None,
        headers: HashMap::new(),
        body: body.as_bytes().to_vec(),
    };
    router.handle(&request, "127.0.0.1:1".parse().unwrap())
}

fn parse(response: &Response) -> Value {
    json::parse(std::str::from_utf8(&response.body).unwrap()).unwrap()
}

#[test]
fn health_reports_the_content_counts() {
    let response = get(&router(), "/health");
    assert_eq!(response.status, Status::Ok);
    let body = parse(&response);
    assert_eq!(body.get("status").unwrap().as_str(), Some("ok"));
    assert!(body.get("documents").unwrap().as_f64().unwrap() > 0.0);
}

#[test]
fn document_routes_serve_content() {
    let router = router();
    let index = parse(&get(&router, "/api/docs"));
    let groups = index.get("groups").unwrap().as_array();
    assert_eq!(groups.len(), 2, "the concepts and the implementation");
    let first = groups[0]
        .get("sections")
        .unwrap()
        .as_array()
        .first()
        .unwrap()
        .get("documents")
        .unwrap()
        .as_array()
        .first()
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let document = parse(&get(&router, &format!("/api/docs/{first}")));
    let forms = document.get("forms").unwrap().as_array();
    assert!(!forms.is_empty());
    assert!(
        forms[0]
            .get("html")
            .unwrap()
            .as_str()
            .unwrap()
            .contains('<')
    );
}

#[test]
fn documents_carry_both_languages() {
    let router = router();
    let document = parse(&get(&router, "/api/docs/sgp4"));
    let forms = document.get("forms").unwrap().as_array();
    assert_eq!(forms.len(), 2, "the sample document has a translation");
    let languages: Vec<&str> = forms
        .iter()
        .map(|form| form.get("language").unwrap().as_str().unwrap())
        .collect();
    assert!(languages.contains(&"zh"));
    assert!(languages.contains(&"en"));
}

#[test]
fn document_listing_carries_bilingual_titles() {
    let router = router();
    let index = parse(&get(&router, "/api/docs"));
    let groups = index.get("groups").unwrap().as_array();
    let title = groups[0].get("title").unwrap();
    assert!(title.get("zh").unwrap().as_str().is_some());
    assert!(title.get("en").unwrap().as_str().is_some());
    let section = groups[0]
        .get("sections")
        .unwrap()
        .as_array()
        .first()
        .unwrap();
    assert!(
        section
            .get("title")
            .unwrap()
            .get("zh")
            .unwrap()
            .as_str()
            .is_some()
    );
}

#[test]
fn search_returns_ranked_hits() {
    let response = get(&router(), "/api/docs/search?q=propagation");
    let body = parse(&response);
    assert!(body.get("count").unwrap().as_f64().unwrap() > 0.0);
}

#[test]
fn graph_routes_serve_the_concept_and_crate_graphs() {
    let router = router();
    let concepts = parse(&get(&router, "/api/graph"));
    assert!(!concepts.get("nodes").unwrap().as_array().is_empty());
    let crates = parse(&get(&router, "/api/graph/crates"));
    assert!(!crates.get("nodes").unwrap().as_array().is_empty());
}

#[test]
fn api_catalog_routes_serve_items() {
    let router = router();
    let index = parse(&get(&router, "/api/api"));
    assert!(!index.get("crates").unwrap().as_array().is_empty());
    let items = index.get("items").unwrap().as_array();
    let name = items
        .first()
        .unwrap()
        .get("name")
        .unwrap()
        .as_str()
        .unwrap();
    let item = parse(&get(&router, &format!("/api/api/items/{name}")));
    assert!(item.get("signature").is_some());
}

#[test]
fn tool_registry_and_execution() {
    let router = router();
    let registry = parse(&get(&router, "/api/tools"));
    assert!(registry.get("tools").unwrap().as_array().len() >= 7);
    let response = post(&router, "/api/tools/sgp4-propagate/run", r#"{"minutes":0}"#);
    let body = parse(&response);
    assert_eq!(body.get("ok").unwrap().as_bool(), Some(true));
}

#[test]
fn tool_failures_are_reported_with_the_parameter() {
    let router = router();
    let response = post(
        &router,
        "/api/tools/sgp4-propagate/run",
        r#"{"minutes":100000000}"#,
    );
    let body = parse(&response);
    assert_eq!(body.get("ok").unwrap().as_bool(), Some(false));
    assert_eq!(
        body.get("error")
            .unwrap()
            .get("parameter")
            .unwrap()
            .as_str(),
        Some("minutes")
    );
}

#[test]
fn malformed_bodies_are_rejected() {
    let router = router();
    let response = post(&router, "/api/tools/sgp4-propagate/run", "{");
    let body = parse(&response);
    assert_eq!(body.get("ok").unwrap().as_bool(), Some(false));
}

#[test]
fn a_hashed_asset_is_immutable_and_a_plain_one_is_not() {
    // A content hash in the name means the name changes with the
    // contents, so the browser can keep it for as long as it likes.

    // 名字中含内容哈希意味着名字随内容变化，因此浏览器可以想存多久就存多久。
    assert_eq!(
        cache_policy("assets/index-Vou83oaV.js"),
        "public, max-age=31536000, immutable"
    );
    assert_eq!(
        cache_policy("assets/style-CEH25aoZ.css"),
        "public, max-age=31536000, immutable"
    );
    // A name without one is reused by the next build.
    // 不含哈希的名字会被下一次构建复用。
    assert_eq!(cache_policy("index.html"), UNHASHED_CACHE_POLICY);
    assert_eq!(cache_policy("assets/app.js"), UNHASHED_CACHE_POLICY);
    assert_eq!(cache_policy("favicon.ico"), UNHASHED_CACHE_POLICY);
    // A name that merely ends in a dash is not hashed.
    // 仅仅以连字符结尾的名字不算带哈希。
    assert_eq!(cache_policy("assets/app-.js"), UNHASHED_CACHE_POLICY);
    assert_eq!(cache_policy("assets/no-extension"), UNHASHED_CACHE_POLICY);
}

#[test]
fn a_hash_is_recognised_by_its_shape() {
    // The bundler emits a mixed case base64 hash.
    // 打包器输出的哈希是大小写混合的 base64。
    assert!(is_content_hashed("index-Vou83oaV.js"));
    assert!(is_content_hashed("style-CEH25aoZ.css"));
    assert!(is_content_hashed("app-deadbeef.js"));
    assert!(!is_content_hashed("app-abc.js"), "too few digits");
    assert!(!is_content_hashed("app-na-me.js"), "a dash is not a hash");
    assert!(!is_content_hashed("app.js"), "no hash segment");
    assert!(!is_content_hashed("noextension"));
}

#[test]
fn unknown_routes_fall_back_to_the_entry_point() {
    let router = router();
    let response = get(&router, "/some/client/route");
    assert_eq!(response.status, Status::Ok);
    assert!(response.content_type.starts_with("text/html"));
}

#[test]
fn unknown_api_resources_report_not_found() {
    let router = router();
    let response = get(&router, "/api/docs/not-a-document");
    assert_eq!(response.status, Status::NotFound);
    let body = parse(&response);
    assert_eq!(body.get("ok").unwrap().as_bool(), Some(false));
}

#[test]
fn post_to_a_get_route_is_refused() {
    let router = router();
    assert_eq!(
        post(&router, "/api/docs", "{}").status,
        Status::MethodNotAllowed
    );
}

#[test]
fn a_crate_card_takes_the_first_sentence_only() {
    // The index shows what a crate is; the crate page shows the rest.
    // 索引说明一个 crate 是什么；其余内容由 crate 页面呈现。
    assert_eq!(
        crate_summary_brief(
            "ApsisRS \u{7684}\u{5171}\u{4eab}\u{57fa}\u{7840}\u{7c7b}\u{578b}\u{3002}\u{672c} crate \u{4f4d}\u{4e8e}\u{4f9d}\u{8d56}\u{56fe}\u{7684}\u{5e95}\u{5c42}\u{3002}"
        ),
        "ApsisRS \u{7684}\u{5171}\u{4eab}\u{57fa}\u{7840}\u{7c7b}\u{578b}\u{3002}"
    );
}

#[test]
fn a_number_is_not_a_sentence_end() {
    // A version or a measurement must not cut a sentence in half.
    // 版本号或量值不得把句子截成两半。
    assert_eq!(
        crate_summary_brief("Reads version 1.5 of the format. Then it checksums."),
        "Reads version 1.5 of the format."
    );
}

#[test]
fn a_summary_without_a_full_stop_is_capped() {
    let long = "word ".repeat(80);
    let brief = crate_summary_brief(&long);
    assert!(brief.chars().count() <= 158, "the brief is too long");
    assert!(brief.ends_with('\u{2026}'), "it should be marked as cut");
}
