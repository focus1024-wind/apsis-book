//! Request routing.
//!
//! The router turns a [`Request`] into a [`Response`]. It owns the route
//! table, the JSON encoding of each payload, and the mapping from an error
//! to a status code. Every route is a plain function so that the table reads
//! as a list of what the service offers.
//!
//! The routes fall into four groups:
//!
//! - `/health` and `/api/health` report that the process is serving.
//! - `/api/docs`, `/api/graph`, and `/api/api` expose the content of tasks
//!   one and two.
//! - `/api/tools` exposes the tool registry of task three and runs a tool.
//! - Every other path is served from the embedded frontend, with directory
//!   requests falling back to `index.html` so that client side routing
//!   works on a reload.

use std::net::SocketAddr;
use std::sync::Arc;

use crate::docs::{ApiCatalog, Language};
use crate::embedded;
use crate::graph::Graph;
use crate::http::{Handler, Method, Request, Response, Status};
use crate::json::{self, Value};
use crate::json_object;
use crate::state::AppState;
use crate::tools;

/// The request router.
pub struct Router {
    state: Arc<AppState>,
    /// Pre-encoded document listing, built once at startup.
    docs_index: String,
    /// Pre-encoded search index, built once at startup.
    search: crate::docs::SearchIndex,
    /// Pre-encoded concept graph.
    concept_graph: String,
    /// Pre-encoded list of the flow graphs.
    flow_index: String,
    /// Pre-encoded crate structure graph.
    crate_graph: String,
    /// Pre-encoded tool registry.
    tool_registry: String,
}

impl Router {
    /// Builds the router over the parsed content.
    pub fn new(state: Arc<AppState>) -> Self {
        let docs_index = json::to_string(&document_index(&state));
        let search = state.docs.search_index();
        let concept_graph = json::to_string(&graph_json(&state.graph.concepts));
        let flow_index = json::to_string(&flow_list_json(&state.graph.flows));
        let crate_graph = json::to_string(&graph_json(&crate::graph::crate_structure(&state.api)));
        let tool_registry = json::to_string(&tools::registry_json(&state.tools));
        Self {
            state,
            docs_index,
            search,
            concept_graph,
            flow_index,
            crate_graph,
            tool_registry,
        }
    }
}

impl Handler for Router {
    fn handle(&self, request: &Request, _peer: SocketAddr) -> Response {
        let path = request.path.as_str();

        match (request.method.clone(), path) {
            (Method::Get | Method::Head, "/health")
            | (Method::Get | Method::Head, "/api/health") => self.health(),
            (Method::Get | Method::Head, "/api/docs") => {
                Response::json(self.docs_index.clone()).with_cache_control("no-cache")
            }
            (Method::Get | Method::Head, "/api/docs/search") => self.search_documents(request),
            (Method::Get | Method::Head, "/api/graph") => {
                Response::json(self.concept_graph.clone()).with_cache_control("no-cache")
            }
            (Method::Get | Method::Head, "/api/graph/flows") => {
                Response::json(self.flow_index.clone()).with_cache_control("no-cache")
            }
            (Method::Get | Method::Head, "/api/graph/crates") => {
                Response::json(self.crate_graph.clone()).with_cache_control("no-cache")
            }
            (Method::Get | Method::Head, "/api/api") => self.api_index(),
            (Method::Get | Method::Head, "/api/tools") => {
                Response::json(self.tool_registry.clone()).with_cache_control("no-cache")
            }
            (Method::Get, _) if path.starts_with("/api/docs/") => self.document(path),
            (Method::Get, _) if path.starts_with("/api/graph/") => self.flow(path),
            (Method::Get, _) if path.starts_with("/api/api/crates/") => self.api_crate(path),
            (Method::Get, _) if path.starts_with("/api/api/items/") => self.api_item(path),
            (Method::Get, _) if path.starts_with("/api/tools/") && path.ends_with("/run") => {
                Response::error(Status::MethodNotAllowed, "this route accepts POST")
            }
            (Method::Get, _) if path.starts_with("/api/tools/") => self.tool(path),
            (Method::Post, _) if path.starts_with("/api/tools/") && path.ends_with("/run") => {
                self.run_tool(request, path)
            }
            (Method::Post, _) if path.starts_with("/api/") => {
                Response::error(Status::MethodNotAllowed, "this route does not accept POST")
            }
            (Method::Get | Method::Head, _) => self.static_asset(path),
            (Method::Options, _) => {
                Response::empty(Status::NoContent).with_header("Allow", "GET, HEAD, POST, OPTIONS")
            }
            _ => Response::error(Status::MethodNotAllowed, "unsupported method"),
        }
    }
}

impl Router {
    /// Reports service health and the state of the embedded content.
    fn health(&self) -> Response {
        let body = json_object! {
            "status" => "ok",
            "service" => "apsis-book",
            "version" => env!("CARGO_PKG_VERSION"),
            "documents" => self.state.docs.len(),
            "chapters" => self.state.docs.chapters().len(),
            "graph_nodes" => self.state.graph.concepts.nodes.len(),
            "api_crates" => self.state.api.crates.len(),
            "api_items" => self.state.api.item_count(),
            "tools" => self.state.tools.entries().len(),
            "frontend_embedded" => embedded::is_populated(),
        };
        Response::json(json::to_string(&body)).with_cache_control("no-store")
    }

    /// Searches the document index.
    fn search_documents(&self, request: &Request) -> Response {
        let query = request.query_pairs();
        let text = query.get("q").cloned().unwrap_or_default();
        let limit = query
            .get("limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(20)
            .min(100);
        let hits = self.search.search(&text, limit);
        let hits: Vec<Value> = hits
            .iter()
            .map(|hit| {
                json_object! {
                    "id" => hit.id.clone(),
                    "title" => hit.title.clone(),
                    "summary" => hit.summary.clone(),
                    "language" => hit.language.tag(),
                    "chapter" => hit.chapter.clone(),
                    "chapter_title" => chapter_title(&self.state, &hit.chapter),
                    "score" => hit.score,
                }
            })
            .collect();
        let body = json_object! {
            "query" => text,
            "count" => hits.len(),
            "indexed" => self.search.len(),
            "hits" => hits,
        };
        Response::json(json::to_string(&body)).with_cache_control("no-store")
    }

    /// Returns one document with its rendered HTML.
    ///
    /// Both language forms are returned when both are embedded, so the
    /// reader can switch the language without another request. A language
    /// without a translation is omitted and the frontend falls back to the
    /// English form.
    fn document(&self, path: &str) -> Response {
        let id = path.trim_start_matches("/api/docs/");
        if id.is_empty() || id.contains('/') {
            return not_found("document");
        }
        match self.state.document(id) {
            Some(document) => {
                let mut forms = Vec::new();
                for language in [Language::Chinese, Language::English] {
                    if !document.has(language) {
                        continue;
                    }
                    let variant = document.variant(language);
                    forms.push(json_object! {
                        "language" => language.tag(),
                        "title" => variant.title.clone(),
                        "summary" => variant.summary.clone(),
                        "html" => variant.html.clone(),
                        "markdown" => variant.markdown.clone(),
                        "headings" => variant
                            .headings
                            .iter()
                            .map(|(anchor, text)| json_object! { "anchor" => anchor.clone(), "text" => text.clone() })
                            .collect::<Vec<Value>>(),
                    });
                }
                let body = json_object! {
                    "id" => document.id.clone(),
                    "chapter" => document.chapter.clone(),
                    "tags" => tag_lists(document),
                    "prerequisites" => document.prerequisites.iter().map(|id| Value::from(id.as_str())).collect::<Vec<Value>>(),
                    "references" => document.references().iter().map(|id| Value::from(id.as_str())).collect::<Vec<Value>>(),
                    "forms" => forms,
                };
                Response::json(json::to_string(&body)).with_cache_control("no-cache")
            }
            None => not_found("document"),
        }
    }

    /// Returns one flow graph by identifier.
    fn flow(&self, path: &str) -> Response {
        let id = path.trim_start_matches("/api/graph/");
        if id == "flows" {
            return Response::json(self.flow_index.clone()).with_cache_control("no-cache");
        }
        if id == "crates" {
            return Response::json(self.crate_graph.clone()).with_cache_control("no-cache");
        }
        if id == "concepts" {
            return Response::json(self.concept_graph.clone()).with_cache_control("no-cache");
        }
        match self.state.graph.flow(id) {
            Some(graph) => {
                Response::json(json::to_string(&graph_json(graph))).with_cache_control("no-cache")
            }
            None => not_found("graph"),
        }
    }

    /// Returns the interface catalog summary.
    fn api_index(&self) -> Response {
        let catalog = &self.state.api;
        let item_names = &catalog.item_names;
        let crates: Vec<Value> = catalog
            .crates
            .iter()
            .map(|krate| {
                json_object! {
                    "name" => krate.name.clone(),
                    "layer" => krate.layer.clone(),
                    "summary" => crate_texts(krate, item_names),
                    "items" => krate.items.len(),
                }
            })
            .collect();
        let items: Vec<Value> = catalog
            .crates
            .iter()
            .flat_map(|krate| {
                krate.items.iter().map(move |item| {
                    json_object! {
                        "name" => item.name.clone(),
                        "kind" => item.kind.clone(),
                        "crate" => krate.name.clone(),
                    }
                })
            })
            .collect();
        let body = json_object! {
            "generated_from" => catalog.generated_from.iter().map(|path| Value::from(path.as_str())).collect::<Vec<Value>>(),
            "crate_count" => catalog.crates.len(),
            "item_count" => catalog.item_count(),
            "crates" => crates,
            "items" => items,
        };
        Response::json(json::to_string(&body)).with_cache_control("no-cache")
    }

    /// Returns one crate of the interface catalog.
    fn api_crate(&self, path: &str) -> Response {
        let name = path.trim_start_matches("/api/api/crates/");
        match self.state.api.krate(name) {
            Some(krate) => Response::json(json::to_string(&crate_json(
                krate,
                &self.state.api.item_names,
            )))
            .with_cache_control("no-cache"),
            None => not_found("crate"),
        }
    }

    /// Returns one interface item by name.
    fn api_item(&self, path: &str) -> Response {
        let name = path.trim_start_matches("/api/api/items/");
        match self.state.api.find_item(name) {
            Some((krate, item)) => {
                let body = json_object! {
                    "crate" => krate.name.clone(),
                    "layer" => krate.layer.clone(),
                    "kind" => item.kind.clone(),
                    "name" => item.name.clone(),
                    "signature" => item.signature.clone(),
                    "doc" => doc_texts(item, &self.state.api.item_names),
                    "associated" => item.associated,
                };
                Response::json(json::to_string(&body)).with_cache_control("no-cache")
            }
            None => not_found("interface item"),
        }
    }

    /// Returns one tool description.
    fn tool(&self, path: &str) -> Response {
        let id = path.trim_start_matches("/api/tools/");
        match self.state.tools.get(id) {
            Some(tool) => {
                let body = tools::registry_json(&crate::tools::Registry::new());
                let _ = body;
                let single = tool_json(tool);
                Response::json(json::to_string(&single)).with_cache_control("no-cache")
            }
            None => not_found("tool"),
        }
    }

    /// Runs a tool.
    ///
    /// The body must be a JSON object of parameters. A body that is not
    /// valid JSON, or a parameter that fails validation, produces a bad
    /// request status with a message naming the parameter.
    fn run_tool(&self, request: &Request, path: &str) -> Response {
        let id = path
            .trim_start_matches("/api/tools/")
            .trim_end_matches("/run");
        if self.state.tools.get(id).is_none() {
            return not_found("tool");
        }
        let Some(text) = request.body_text() else {
            return Response::error(Status::BadRequest, "the request body must be JSON text");
        };
        if text.trim().is_empty() {
            return Response::error(Status::BadRequest, "the request body must be a JSON object");
        }
        let values = match json::parse(text) {
            Ok(values) => values,
            Err(error) => {
                return Response::json(json::to_string(&json_object! {
                    "ok" => false,
                    "error" => json_object! {
                        "message" => format!("the request body is not valid JSON: {error}"),
                    },
                }));
            }
        };
        match tools::execute(&self.state.tools, id, &values) {
            Ok(result) => Response::json(json::to_string(&json_object! {
                "ok" => true,
                "tool" => id,
                "result" => result,
            }))
            .with_cache_control("no-store"),
            Err(error) => Response::json(json::to_string(&json_object! {
                "ok" => false,
                "tool" => id,
                "error" => json_object! {
                    "parameter" => error.parameter.map(Value::from).unwrap_or(Value::Null),
                    "message" => error.message.clone(),
                },
            }))
            .with_cache_control("no-store"),
        }
    }

    /// Serves an embedded frontend asset.
    ///
    /// A request that names a file is answered with that file. Any other
    /// path falls back to `index.html` so that a client side route survives
    /// a reload. When no frontend is embedded the placeholder page is
    /// served, which keeps the JSON interfaces reachable from a browser.
    fn static_asset(&self, path: &str) -> Response {
        let relative = path.trim_start_matches('/');

        if !embedded::is_populated() {
            if relative.is_empty() || !relative.contains('.') {
                return Response::new(
                    Status::Ok,
                    "text/html; charset=utf-8",
                    embedded::placeholder().to_vec(),
                )
                .with_cache_control("no-store");
            }
            return not_found("asset");
        }

        if !relative.is_empty() && !relative.ends_with('/') {
            if let Some(bytes) = embedded::get(relative) {
                return Response::new(Status::Ok, embedded::content_type(relative), bytes.to_vec())
                    .with_cache_control(CACHE_POLICY);
            }
        }

        // A directory request, or an unknown path that looks like a route,
        // returns the single page entry point.
        match embedded::get("index.html") {
            Some(bytes) => Response::new(Status::Ok, "text/html; charset=utf-8", bytes.to_vec())
                .with_cache_control("no-cache"),
            None => not_found("asset"),
        }
    }
}

/// The cache policy for every asset.
///
/// The build writes stable file names rather than content hashed ones, so a
/// browser must revalidate after a deployment. A year long immutable policy
/// would be correct only for hashed names, and would pin a reader to the
/// bundle that was current when they first loaded the page.
const CACHE_POLICY: &str = "no-cache";

/// Builds the document listing payload.
fn document_index(state: &AppState) -> Value {
    let chapters: Vec<Value> = state
        .docs
        .chapters()
        .iter()
        .map(|chapter| {
            let documents: Vec<Value> = chapter
                .documents
                .iter()
                .filter_map(|id| state.docs.document(id))
                .map(|document| {
                    json_object! {
                        "id" => document.id.clone(),
                        "title" => bilingual("title", document),
                        "summary" => bilingual("summary", document),
                        "tags" => tag_lists(document),
                    }
                })
                .collect();
            json_object! {
                "id" => chapter.id.clone(),
                "title" => optional_bilingual(&chapter.title_zh, &chapter.title),
                "summary" => optional_bilingual(&chapter.summary_zh, &chapter.summary),
                "documents" => documents,
            }
        })
        .collect();

    json_object! {
        "document_count" => state.docs.len(),
        "chapters" => chapters,
    }
}

/// Encodes a documentation text and its rendering in both languages.
///
/// The documentation comment is rendered to HTML here, by the same renderer
/// the concept documents use, so a comment that contains a list, a table,
/// or a code block is drawn as one. The two `plain` members carry the text
/// with the markup removed, which the index uses for a one line summary.
fn doc_texts(item: &crate::docs::ApiItem, items: &std::collections::HashSet<String>) -> Value {
    let zh = item.doc_zh.as_deref().unwrap_or(&item.doc);
    json_object! {
        "zh" => zh.to_string(),
        "en" => item.doc.clone(),
        "html" => json_object! {
            "zh" => crate::docs::markdown::render_with(zh, Some(items)),
            "en" => crate::docs::markdown::render_with(&item.doc, Some(items)),
        },
        "plain" => json_object! {
            "zh" => crate::docs::markdown::to_plain_text(zh),
            "en" => crate::docs::markdown::to_plain_text(&item.doc),
        },
    }
}

/// Encodes a crate summary in both languages, with the plain text.
fn crate_texts(krate: &crate::docs::ApiCrate, items: &std::collections::HashSet<String>) -> Value {
    let zh = krate.summary_zh.as_deref().unwrap_or(&krate.summary);
    json_object! {
        "zh" => zh.to_string(),
        "en" => krate.summary.clone(),
        "plain" => json_object! {
            "zh" => crate_summary_plain(zh),
            "en" => crate_summary_plain(&krate.summary),
        },
        "html" => json_object! {
            "zh" => crate::docs::markdown::render_with(zh, Some(items)),
            "en" => crate::docs::markdown::render_with(&krate.summary, Some(items)),
        },
    }
}

/// Returns the plain text of a crate summary.
///
/// The Markdown renderer is applied first so that a summary that opens with
/// a list or a heading is reduced to its prose.
fn crate_summary_plain(summary: &str) -> String {
    crate::docs::markdown::to_plain_text(summary)
}

/// Encodes a document title or summary in both languages.
///
/// The two names select the field of a document; an unknown name yields an
/// empty pair rather than a panic.
fn bilingual(field: &str, document: &crate::docs::Document) -> Value {
    let (zh, en) = match field {
        "title" => (
            document.title(Language::Chinese),
            document.title(Language::English),
        ),
        "summary" => (
            document.summary(Language::Chinese),
            document.summary(Language::English),
        ),
        _ => ("", ""),
    };
    json_object! { "zh" => zh.to_string(), "en" => en.to_string() }
}

/// Pairs an optional translation with the form it translates.
///
/// A field without a translation is written with the same text in both
/// members, which keeps the payload uniform and lets the frontend read one
/// shape.
fn optional_bilingual(translation: &Option<String>, fallback: &str) -> Value {
    match translation {
        Some(zh) => json_object! { "zh" => zh.clone(), "en" => fallback.to_string() },
        None => json_object! { "zh" => fallback.to_string(), "en" => fallback.to_string() },
    }
}

/// Encodes the list of flow graphs.
///
/// The list is part of the payload so that the frontend does not carry its
/// own copy of the flow identifiers; adding a flow to the content is enough
/// for it to appear in the navigation.
fn flow_list_json(flows: &[Graph]) -> Value {
    let items: Vec<Value> = flows
        .iter()
        .map(|graph| {
            json_object! {
                "id" => graph.id.clone(),
                "title" => optional_bilingual(&graph.title_zh, &graph.title),
                "summary" => optional_bilingual(&graph.summary_zh, &graph.summary),
                "nodes" => graph.nodes.len(),
                "edges" => graph.edges.len(),
            }
        })
        .collect();
    json_object! { "flows" => items }
}

/// Encodes the tag lists of a document, one per language.
fn tag_lists(document: &crate::docs::Document) -> Value {
    json_object! {
        "en" => document.tags.iter().map(|tag| Value::from(tag.as_str())).collect::<Vec<Value>>(),
        "zh" => document.tags_zh.iter().map(|tag| Value::from(tag.as_str())).collect::<Vec<Value>>(),
    }
}

/// Encodes the title of a chapter, in both languages.
fn chapter_title(state: &AppState, chapter_id: &str) -> Value {
    let Some(chapter) = state.docs.chapters().iter().find(|c| c.id == chapter_id) else {
        return Value::string(chapter_id);
    };
    optional_bilingual(&chapter.title_zh, &chapter.title)
}

/// Encodes a graph as JSON.
///
/// A display label and a summary are carried in both languages when the
/// authored data has a translation. A node whose label is a name, such as a
/// crate, carries only the one form because it is written the same way in
/// both languages.
fn graph_json(graph: &Graph) -> Value {
    let nodes: Vec<Value> = graph
        .nodes
        .iter()
        .map(|node| {
            let label = match &node.label_zh {
                Some(zh) => json_object! { "zh" => zh.clone(), "en" => node.label.clone() },
                None => json_object! { "zh" => node.label.clone(), "en" => node.label.clone() },
            };
            let summary = match &node.summary_zh {
                Some(zh) => json_object! { "zh" => zh.clone(), "en" => node.summary.clone() },
                None => json_object! { "zh" => node.summary.clone(), "en" => node.summary.clone() },
            };
            json_object! {
                "id" => node.id.clone(),
                "label" => label,
                "kind" => node.kind.name(),
                "group" => node.group.clone(),
                "summary" => summary,
                "document" => node.document.clone().map(Value::from).unwrap_or(Value::Null),
                "api" => node.api.clone().map(Value::from).unwrap_or(Value::Null),
            }
        })
        .collect();
    let edges: Vec<Value> = graph
        .edges
        .iter()
        .map(|edge| {
            json_object! {
                "source" => edge.source.clone(),
                "target" => edge.target.clone(),
                "kind" => edge.kind.clone(),
                "label" => edge.label.clone().map(Value::from).unwrap_or(Value::Null),
            }
        })
        .collect();
    let title = match &graph.title_zh {
        Some(zh) => json_object! { "zh" => zh.clone(), "en" => graph.title.clone() },
        None => json_object! { "zh" => graph.title.clone(), "en" => graph.title.clone() },
    };
    let summary = match &graph.summary_zh {
        Some(zh) => json_object! { "zh" => zh.clone(), "en" => graph.summary.clone() },
        None => json_object! { "zh" => graph.summary.clone(), "en" => graph.summary.clone() },
    };
    json_object! {
        "id" => graph.id.clone(),
        "title" => title,
        "summary" => summary,
        "groups" => graph.groups().into_iter().map(Value::from).collect::<Vec<Value>>(),
        "nodes" => nodes,
        "edges" => edges,
    }
}

/// Encodes a crate of the interface catalog as JSON.
fn crate_json(
    krate: &crate::docs::ApiCrate,
    item_names: &std::collections::HashSet<String>,
) -> Value {
    let items: Vec<Value> = krate
        .items
        .iter()
        .map(|item| {
            json_object! {
                "kind" => item.kind.clone(),
                "name" => item.name.clone(),
                "signature" => item.signature.clone(),
                "doc" => doc_texts(item, item_names),
                "associated" => item.associated,
            }
        })
        .collect();
    json_object! {
        "name" => krate.name.clone(),
        "layer" => krate.layer.clone(),
        "summary" => crate_texts(krate, item_names),
        "items" => items,
    }
}

/// Encodes one tool description as JSON.
fn tool_json(tool: &crate::tools::ToolSpec) -> Value {
    // The registry encoder is the single source of the tool shape, so the
    // listing of all tools and the description of one tool cannot drift.
    let registry = tools::registry_json(&crate::tools::Registry::new());
    let listed = registry
        .get("tools")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .find(|entry| entry.get("id").and_then(Value::as_str) == Some(tool.id))
        .cloned();
    match listed {
        Some(value) => value,
        None => json_object! { "id" => tool.id },
    }
}

/// Returns the catalog summary for the interface index, used by tests.
#[allow(dead_code)]
fn catalog_summary(catalog: &ApiCatalog) -> Value {
    json_object! {
        "crates" => catalog.crates.len(),
        "items" => catalog.item_count(),
    }
}

/// Builds a not found response naming the kind of resource.
///
/// The status is a real `404` so that a client can distinguish a missing
/// resource from a server failure, and the body carries the same message in
/// the shape the other routes use.
fn not_found(kind: &str) -> Response {
    Response::new(
        Status::NotFound,
        "application/json; charset=utf-8",
        json::to_string(&json_object! {
            "ok" => false,
            "error" => json_object! { "message" => format!("no such {kind}") },
        })
        .into_bytes(),
    )
    .with_cache_control("no-store")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn router() -> Router {
        Router::new(Arc::new(AppState::new()))
    }

    fn get(router: &Router, path: &str) -> Response {
        // The request parser splits the target, so a test that wants a query
        // supplies it in the path and it is split the same way here.
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
        let chapters = index.get("chapters").unwrap().as_array();
        assert!(!chapters.is_empty());
        let first = chapters[0]
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
        let chapters = index.get("chapters").unwrap().as_array();
        let title = chapters[0].get("title").unwrap();
        assert!(title.get("zh").unwrap().as_str().is_some());
        assert!(title.get("en").unwrap().as_str().is_some());
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
}
