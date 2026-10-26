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
//!
//! 请求路由。
//!
//! 路由器把一个 [`Request`] 变成一个 [`Response`]。它拥有路由表、各载荷的 JSON
//! 编码，以及从错误到状态码的映射。每个路由都是一个普通函数，使这张表读起来就是
//! 服务所提供内容的一份清单。
//!
//! 路由分为四组：
//!
//! - `/health` 与 `/api/health` 报告进程正在提供服务。
//! - `/api/docs`、`/api/graph` 与 `/api/api` 暴露第一、二项任务的内容。
//! - `/api/tools` 暴露第三项任务的工具注册表并运行工具。
//! - 其余路径都由内嵌前端提供，其中目录请求回退到 `index.html`，使客户端路由在
//!   刷新后仍然工作。

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
///
/// 请求路由器。
pub struct Router {
    state: Arc<AppState>,
    /// Pre-encoded document listing, built once at startup.
    ///
    /// 预先编码的文档列表，在启动时构建一次。
    docs_index: String,
    /// Pre-encoded search index, built once at startup.
    ///
    /// 预先编码的搜索索引，在启动时构建一次。
    search: crate::docs::SearchIndex,
    /// Pre-encoded concept graph.
    ///
    /// 预先编码的概念图谱。
    concept_graph: String,
    /// Pre-encoded list of the flow graphs.
    ///
    /// 预先编码的流程图列表。
    flow_index: String,
    /// Pre-encoded module structure graph.
    ///
    /// 预先编码的模块结构图。
    module_graph: String,
    /// Pre-encoded tool registry.
    ///
    /// 预先编码的工具注册表。
    tool_registry: String,
}

impl Router {
    /// Builds the router over the parsed content.
    ///
    /// 在已解析的内容之上构建路由器。
    pub fn new(state: Arc<AppState>) -> Self {
        let docs_index = json::to_string(&document_index(&state));
        let search = state.docs.search_index();
        let concept_graph = json::to_string(&graph_json(&state.graph.concepts));
        let flow_index = json::to_string(&flow_list_json(&state.graph.flows));
        let module_graph =
            json::to_string(&graph_json(&crate::graph::module_structure(&state.api)));
        let tool_registry = json::to_string(&tools::registry_json(&state.tools));
        Self {
            state,
            docs_index,
            search,
            concept_graph,
            flow_index,
            module_graph,
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
            (Method::Get | Method::Head, "/api/graph/modules") => {
                Response::json(self.module_graph.clone()).with_cache_control("no-cache")
            }
            (Method::Get | Method::Head, "/api/api") => self.api_index(),
            (Method::Get | Method::Head, "/api/tools") => {
                Response::json(self.tool_registry.clone()).with_cache_control("no-cache")
            }
            (Method::Get, _) if path.starts_with("/api/docs/") => self.document(path),
            (Method::Get, _) if path.starts_with("/api/graph/") => self.flow(path),
            (Method::Get, _) if path.starts_with("/api/api/modules/") => self.api_module(path),
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
    ///
    /// 报告服务健康状况与内嵌内容的状态。
    fn health(&self) -> Response {
        let body = json_object! {
            "status" => "ok",
            "service" => "apsis-book",
            "version" => env!("CARGO_PKG_VERSION"),
            "documents" => self.state.docs.len(),
            "layers" => self
                .state
                .docs
                .groups()
                .iter()
                .map(|group| group.sections.len())
                .sum::<usize>(),
            "graph_nodes" => self.state.graph.concepts.nodes.len(),
            "api_modules" => self.state.api.modules.len(),
            "api_items" => self.state.api.item_count(),
            "tools" => self.state.tools.entries().len(),
            "frontend_embedded" => embedded::is_populated(),
        };
        Response::json(json::to_string(&body)).with_cache_control("no-store")
    }

    /// Searches the document index.
    ///
    /// 搜索文档索引。
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
    ///
    /// 返回一篇文档及其渲染后的 HTML。
    ///
    /// 两种语言形式都嵌入时都被返回，使读者无需再次请求即可切换语言。没有译文的语言
    /// 被省略，前端回退到英文形式。
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
                            .map(|heading| {
                            json_object! {
                                "anchor" => heading.anchor.clone(),
                                "text" => heading.text.clone(),
                                "level" => heading.level as f64,
                            }
                        })
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
    ///
    /// 按标识返回一张流程图。
    fn flow(&self, path: &str) -> Response {
        let id = path.trim_start_matches("/api/graph/");
        if id == "flows" {
            return Response::json(self.flow_index.clone()).with_cache_control("no-cache");
        }
        if id == "modules" {
            return Response::json(self.module_graph.clone()).with_cache_control("no-cache");
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
    ///
    /// 返回接口目录概要。
    fn api_index(&self) -> Response {
        let catalog = &self.state.api;
        let item_names = &catalog.item_names;
        let modules: Vec<Value> = catalog
            .modules
            .iter()
            .map(|module| {
                json_object! {
                    "name" => module.name.clone(),
                    "layer" => module.layer.clone(),
                    "summary" => module_texts(module, item_names),
                    "items" => module.items.len(),
                }
            })
            .collect();
        let items: Vec<Value> = catalog
            .modules
            .iter()
            .flat_map(|module| {
                module.items.iter().map(move |item| {
                    json_object! {
                        "name" => item.name.clone(),
                        "kind" => item.kind.clone(),
                        "module" => module.name.clone(),
                    }
                })
            })
            .collect();
        let body = json_object! {
            "generated_from" => catalog.generated_from.iter().map(|path| Value::from(path.as_str())).collect::<Vec<Value>>(),
            "module_count" => catalog.modules.len(),
            "item_count" => catalog.item_count(),
            // The layers the modules belong to, in reading order, so that the
            // index is organized the same way as the concept documentation
            // rather than as one flat list.

            // 各模块所属的层，按阅读顺序，使索引的组织方式与概念文档相同，
            // 而不是一张平铺的列表。
            "layers" => api_layers(&self.state.docs),
            "modules" => modules,
            "items" => items,
        };
        Response::json(json::to_string(&body)).with_cache_control("no-cache")
    }

    /// Returns one module of the interface catalog.
    ///
    /// 返回接口目录中的一个模块。
    fn api_module(&self, path: &str) -> Response {
        let name = path.trim_start_matches("/api/api/modules/");
        match self.state.api.module(name) {
            Some(module) => Response::json(json::to_string(&module_json(
                module,
                &self.state.api.item_names,
            )))
            .with_cache_control("no-cache"),
            None => not_found("module"),
        }
    }

    /// Returns one interface item by name.
    ///
    /// 按名字返回一个接口条目。
    fn api_item(&self, path: &str) -> Response {
        let name = path.trim_start_matches("/api/api/items/");
        match self.state.api.find_item(name) {
            Some((module, item)) => {
                let body = json_object! {
                    "module" => module.name.clone(),
                    "layer" => module.layer.clone(),
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
    ///
    /// 返回一个工具的描述。
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
    ///
    /// 运行一个工具。
    ///
    /// 请求体必须是一个参数的 JSON 对象。不是合法 JSON 的请求体，或校验失败的参数，
    /// 会产生一个「请求有误」状态，其消息指名该参数。
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
    ///
    /// 提供一个内嵌的前端资源。
    ///
    /// 指名某个文件的请求以该文件作答。其余任何路径回退到 `index.html`，使客户端路由
    /// 在刷新后存活。未嵌入前端时提供占位页面，使 JSON 接口仍能从浏览器触达。
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
                    .with_cache_control(cache_policy(relative));
            }
        }

        // A directory request, or an unknown path that looks like a route,
        // returns the single page entry point, which names the current
        // bundle and is therefore never served from a cache.

        // 目录请求，或看起来像路由的未知路径，返回单页入口；入口指名当前的包，
        // 因此绝不会从缓存中提供。
        match embedded::get("index.html") {
            Some(bytes) => Response::new(Status::Ok, "text/html; charset=utf-8", bytes.to_vec())
                .with_cache_control(ENTRY_CACHE_POLICY),
            None => not_found("asset"),
        }
    }
}

/// The cache policy for the entry point.
///
/// The entry point names the current bundle, so it is never reused from a
/// cache without asking. `must-revalidate` states that requirement to
/// intermediaries as well as to the browser.
///
/// 入口页面的缓存策略。
///
/// 入口指名当前的包，因此不经询问绝不会从缓存中复用。`must-revalidate` 把这一
/// 要求既向浏览器声明，也向中间代理声明。
const ENTRY_CACHE_POLICY: &str = "no-cache, must-revalidate";

/// The cache policy for a file whose name carries a content hash.
///
/// A build that changes a file also changes its name, so the URL of the new
/// file has never been requested before and cannot be in a cache. Marking it
/// immutable is therefore both safe and free: the browser keeps it until it
/// is no longer named by any entry point.
///
/// 名字带内容哈希的文件的缓存策略。
///
/// 改动某个文件的构建同时改动了它的名字，因此新文件的 URL 从未被请求过，不可能
/// 在缓存里。标记为不可变因而既安全又免费：浏览器会一直保留它，直到不再有任何
/// 入口指名它。
const HASHED_CACHE_POLICY: &str = "public, max-age=31536000, immutable";

/// The cache policy for a file whose name carries no content hash.
///
/// Such a file is a name that a future build reuses, so it must be
/// revalidated rather than reused from a cache.
///
/// 名字不带内容哈希的文件的缓存策略。
///
/// 这样的文件是一个会被后续构建复用的名字，因此必须重新验证，而不能从缓存中复用。
pub(crate) const UNHASHED_CACHE_POLICY: &str = "no-cache, must-revalidate";

/// Returns the cache policy for an asset path.
///
/// The decision is made from the file name alone, which is the same signal
/// the build uses to decide whether the contents changed. Nothing about the
/// contents is read, so the policy is correct for a file that was embedded by
/// an earlier build as well as for one embedded by the current one.
///
/// 返回某个资源路径的缓存策略。
///
/// 判断只依据文件名，而文件名正是构建用来判断内容是否变化的同一个信号。不读取
/// 内容的任何部分，因此该策略对早期构建所嵌入的文件与当前构建所嵌入的文件同样
/// 正确。
pub(crate) fn cache_policy(path: &str) -> &'static str {
    if is_content_hashed(path) {
        HASHED_CACHE_POLICY
    } else {
        UNHASHED_CACHE_POLICY
    }
}

/// Returns `true` when a file name ends with a content hash.
///
/// The build appends the hash before the extension. The alphabet is the one
/// the bundler emits, which is base64 and therefore mixes cases, so the
/// check accepts both and only rejects characters that cannot appear in a
/// hash. A name without one is a name a later build reuses.
///
/// 当文件名以内容哈希结尾时返回 `true`。
///
/// 构建把哈希追加在扩展名之前。其字母表即打包器输出的那一个，是 base64，因而
/// 大小写混用，所以本检查两者都接受，只拒绝不可能出现在哈希中的字符。不含哈希的
/// 名字是后续构建会复用的名字。
pub(crate) fn is_content_hashed(path: &str) -> bool {
    let Some((stem, _)) = path.rsplit_once('.') else {
        return false;
    };
    let Some((_, hash)) = stem.rsplit_once('-') else {
        return false;
    };
    hash.len() >= 8
        && hash
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
}

/// Builds the document listing payload.
///
/// The listing is grouped so that the distinction between the concepts and
/// the implementation survives to the reader: the concepts are the larger
/// superset, and the implementation is what one particular library does.
///
/// 构建文档列表载荷。
///
/// 列表是分组的，使概念与实现之间的区别能传达到读者：概念是更大的超集，而实现
/// 是某一个特定库做了什么。
fn document_index(state: &AppState) -> Value {
    let groups: Vec<Value> = state
        .docs
        .groups()
        .iter()
        .map(|group| {
            let sections: Vec<Value> = group
                .sections
                .iter()
                .map(|section| {
                    let documents = |ids: &[String]| -> Vec<Value> {
                        ids.iter()
                            .filter_map(|id| state.docs.document(id))
                            .map(|document| {
                                json_object! {
                                    "id" => document.id.clone(),
                                    "title" => bilingual("title", document),
                                    "summary" => bilingual("summary", document),
                                    "tags" => tag_lists(document),
                                }
                            })
                            .collect()
                    };
                    // A layer that covers two unrelated subjects carries a
                    // subject for each, rather than one list of documents
                    // that have nothing to do with one another.

                    // 涵盖两个无关主题的层为每个主题各带一个子区，而不是把互不
                    // 相干的文档凑成一份列表。
                    let subsections: Vec<Value> = section
                        .subsections
                        .iter()
                        .map(|subject| {
                            json_object! {
                                "id" => subject.id.clone(),
                                "title" => optional_bilingual(&subject.title_zh, &subject.title),
                                "summary" => optional_bilingual(&subject.summary_zh, &subject.summary),
                                "documents" => documents(&subject.documents),
                            }
                        })
                        .collect();
                    json_object! {
                        "id" => section.id.clone(),
                        "title" => optional_bilingual(&section.title_zh, &section.title),
                        "summary" => optional_bilingual(&section.summary_zh, &section.summary),
                        "documents" => documents(&section.documents),
                        "subsections" => subsections,
                    }
                })
                .collect();
            json_object! {
                "id" => group.id.clone(),
                "title" => optional_bilingual(&group.title_zh, &group.title),
                "sections" => sections,
            }
        })
        .collect();

    json_object! {
        "document_count" => state.docs.len(),
        "groups" => groups,
    }
}

/// Encodes a documentation text and its rendering in both languages.
///
/// 以两种语言编码一段文档文本及其渲染结果。
///
/// The documentation comment is rendered to HTML here, by the same renderer
/// the concept documents use, so a comment that contains a list, a table,
/// or a code block is drawn as one. The two `plain` members carry the text
/// with the markup removed, which the index uses for a one line summary.
///
/// 编码一段文档文本及其两种语言下的渲染结果。
///
/// 文档注释在此用与概念文档相同的渲染器渲染为 HTML，因此含有列表、表格或代码块
/// 的注释会被照原样画出。两个 `plain` 成员携带去掉标记后的文本，索引用它作一行
/// 摘要。
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

/// Encodes a module summary in both languages, with the plain text.
///
/// 以两种语言编码某个模块的概要，并附纯文本。
fn module_texts(
    module: &crate::docs::ApiModule,
    items: &std::collections::HashSet<String>,
) -> Value {
    let zh = module.summary_zh.as_deref().unwrap_or(&module.summary);
    json_object! {
        "zh" => zh.to_string(),
        "en" => module.summary.clone(),
        "plain" => json_object! {
            "zh" => module_summary_plain(zh),
            "en" => module_summary_plain(&module.summary),
        },
        // The index card takes the opening sentence; the module page shows the
        // whole summary under `html`.

        // 索引卡片取首句；模块页面在 `html` 下展示完整概要。
        "brief" => json_object! {
            "zh" => module_summary_brief(zh),
            "en" => module_summary_brief(&module.summary),
        },
        "html" => json_object! {
            "zh" => crate::docs::markdown::render_with(zh, Some(items)),
            "en" => crate::docs::markdown::render_with(&module.summary, Some(items)),
        },
    }
}

/// Returns the plain text of a module summary.
///
/// The Markdown renderer is applied first so that a summary that opens with
/// a list or a heading is reduced to its prose.
///
/// 返回某个模块概要的纯文本。
///
/// 先施加 Markdown 渲染器，使以列表或标题开头的概要归约为其散文。
fn module_summary_plain(summary: &str) -> String {
    crate::docs::markdown::to_plain_text(summary)
}

/// Returns the first sentence of a summary, for a card that links onward.
///
/// 返回概要的首句，供链接到别处的卡片使用。
///
/// Returns the layers of the concept documentation, in reading order.
///
/// The interface catalog records which layer each module belongs to. The index
/// groups the modules by that layer, so it needs the layer's title and its
/// place in the reading order, both of which the concept documentation
/// already declares.
///
/// 返回概念文档的各层，按阅读顺序。
///
/// 接口目录记录了每个模块属于哪一层。索引按该层对模块分组，因此需要层的
/// 标题与它在阅读顺序中的位置，而这两者概念文档都已经声明。
fn api_layers(docs: &crate::docs::Tree) -> Value {
    let layers: Vec<Value> = docs
        .groups()
        .iter()
        .flat_map(|group| group.sections.iter())
        .map(|section| {
            json_object! {
                "id" => section.id.clone(),
                "title" => optional_bilingual(&section.title_zh, &section.title),
                "summary" => optional_bilingual(&section.summary_zh, &section.summary),
            }
        })
        .collect();
    Value::Array(layers)
}

/// Returns the first sentence of a summary, for a card that links onward.
///
/// The module summary is the whole of the module documentation, which runs to
/// several paragraphs and a list of what the module holds. A card on the index
/// shows what a module is, and the module page shows the rest, so the card takes
/// the opening sentence only. Without this the index cards are as tall as the
/// longest module document and the page reads as a wall.
///
/// 返回概要的首句，供链接到别处的卡片使用。
///
/// 模块概要是该模块的整份文档，长达数段并含一份它包含什么的列表。索引上的卡片
/// 说明一个模块是什么，其余由模块页面展示，因此卡片只取首句。没有这一处理，索引
/// 卡片会与最长的模块文档一样高，页面读起来像一堵墙。
pub(crate) fn module_summary_brief(summary: &str) -> String {
    let text = module_summary_plain(summary);
    let text = text.trim();
    // A sentence ends at the first full stop that is not part of a number or
    // an abbreviation, which keeps `1.5` and `e.g.` from cutting a sentence
    // in half.

    // 句子在第一个不属于数字或缩写的句点处结束，这使 `1.5` 与 `e.g.` 不会把
    // 句子截成两半。
    let mut end = None;
    for (index, character) in text.char_indices() {
        // A Chinese sentence ends at its own full stop and is not followed by
        // a space, so those three end a sentence wherever they fall.
        if matches!(character, '\u{3002}' | '\u{ff01}' | '\u{ff1f}') {
            end = Some(index + character.len_utf8());
            break;
        }
        if character == '.' {
            let before = text[..index].chars().next_back();
            let after = text[index + 1..].chars().next();
            let ends_number = before.is_some_and(|c| c.is_ascii_digit())
                && after.is_some_and(|c| c.is_ascii_digit());
            let next_is_space = after.is_none_or(|c| c.is_whitespace());
            if !ends_number && next_is_space {
                end = Some(index + 1);
                break;
            }
        }
    }
    let first = match end {
        Some(index) => &text[..index],
        None => text,
    };
    let first = first.trim();
    // A summary with no full stop would otherwise put a whole paragraph on a
    // card, so the length is capped as well.
    if first.chars().count() <= 160 {
        return first.to_string();
    }
    let mut cut: String = first.chars().take(157).collect();
    cut.push('…');
    cut
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
///
/// 编码流程图列表。
///
/// 该列表是载荷的一部分，使前端无须自备一份流程标识；向内容中新增一张流程图，
/// 就足以让它出现在导航中。
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
///
/// 编码一篇文档的标签列表，每种语言一份。
fn tag_lists(document: &crate::docs::Document) -> Value {
    json_object! {
        "en" => document.tags.iter().map(|tag| Value::from(tag.as_str())).collect::<Vec<Value>>(),
        "zh" => document.tags_zh.iter().map(|tag| Value::from(tag.as_str())).collect::<Vec<Value>>(),
    }
}

/// Encodes the title of a chapter, in both languages.
///
/// 以两种语言编码某个章的标题。
fn chapter_title(state: &AppState, chapter_id: &str) -> Value {
    let found = state
        .docs
        .groups()
        .iter()
        .flat_map(|group| group.sections.iter())
        .find(|section| section.id == chapter_id);
    match found {
        Some(section) => optional_bilingual(&section.title_zh, &section.title),
        None => Value::string(chapter_id),
    }
}

/// Encodes a graph as JSON.
///
/// A display label and a summary are carried in both languages when the
/// authored data has a translation. A node whose label is a name, such as a
/// module, carries only the one form because it is written the same way in
/// both languages.
///
/// 把一张图编码为 JSON。
///
/// 当撰写的数据有译文时，显示标签与概要都以两种语言携带。标签本身是名字的节点
/// ——例如某个模块——只携带一种形式，因为它在两种语言中写法相同。
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
                "label" => optional_bilingual(&edge.label_zh, edge.label.as_deref().unwrap_or("")),
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

/// Encodes a module of the interface catalog as JSON.
///
/// 把接口目录中的一个模块编码为 JSON。
fn module_json(
    module: &crate::docs::ApiModule,
    item_names: &std::collections::HashSet<String>,
) -> Value {
    let items: Vec<Value> = module
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
        "name" => module.name.clone(),
        "layer" => module.layer.clone(),
        "summary" => module_texts(module, item_names),
        "items" => items,
    }
}

/// Encodes one tool description as JSON.
///
/// 把一个工具的描述编码为 JSON。
fn tool_json(tool: &crate::tools::ToolSpec) -> Value {
    // The registry encoder is the single source of the tool shape, so the
    // listing of all tools and the description of one tool cannot drift.

    // 注册表编码器是工具形状的唯一来源，因此全部工具的列表与单个工具的描述不会
    // 漂移。
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
///
/// 返回接口索引的目录概要，供测试使用。
#[allow(dead_code)]
fn catalog_summary(catalog: &ApiCatalog) -> Value {
    json_object! {
        "modules" => catalog.modules.len(),
        "items" => catalog.item_count(),
    }
}

/// Builds a not found response naming the kind of resource.
///
/// The status is a real `404` so that a client can distinguish a missing
/// resource from a server failure, and the body carries the same message in
/// the shape the other routes use.
///
/// 构建一个指名资源种类的「未找到」响应。
///
/// 状态是真正的 `404`，使客户端能把缺失的资源与服务器故障区分开；响应体以其他
/// 路由所用的同一形状携带同一条消息。
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
