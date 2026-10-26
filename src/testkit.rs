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
//!
//! 测试所用的接缝。
//!
//! 服务测试放在与本文件并列的 `tests/`，而不是模块内部，使 `src/` 不含测试代码。
//! 集成测试是独立的 crate，只能触达库公开的部分；若不走接缝，就意味着为了让测试
//! 覆盖一行而把一个内部辅助函数公开。
//!
//! 下列条目在定义处保持 `pub(crate)`，并经由这里的包装函数触达。`pub(crate)` 项
//! 无法再导出为 `pub`，因为 Rust 不允许可见性被放大两次；因此在不把它在所在地
//! 公开的前提下向外提供私有之物，包装函数是唯一的办法。
//!
//! 这里的一切都是所指模块的实现细节，服务不对其稳定性作任何承诺。测试调用的是
//! 包装函数；只要包装函数同步更新，原函数可以自由改变形状。当某个辅助函数不再
//! 值得直接测试时，从这里删除它，而不是把原函数提升为公开。

use crate::docs::api::ApiItem;
use crate::http::request::RequestError;
use crate::http::response::Response;
use crate::json::Value;

/// The cache policy a file without a content hash is served under.
///
/// 名字不含内容哈希的文件所用的缓存策略。
pub const UNHASHED_CACHE_POLICY: &str = crate::api::UNHASHED_CACHE_POLICY;

/// Returns the cache policy for a path.
///
/// 返回某个路径的缓存策略。
pub fn cache_policy(path: &str) -> &'static str {
    crate::api::cache_policy(path)
}

/// Returns `true` when a path carries a content hash in its name.
///
/// 当某个路径的名字中含内容哈希时返回 `true`。
pub fn is_content_hashed(path: &str) -> bool {
    crate::api::is_content_hashed(path)
}

/// Returns the opening sentence of a summary, for a card that links onward.
///
/// 返回概要的首句，供链接到别处的卡片使用。
pub fn module_summary_brief(summary: &str) -> String {
    crate::api::module_summary_brief(summary)
}

/// Parses one interface item of the catalogue.
///
/// 解析目录中的一个接口条目。
pub fn parse_item(value: &Value, module: &str, owner: Option<&str>) -> Option<ApiItem> {
    crate::docs::api::parse_item(value, module, owner)
}

/// Splits text into the tokens the search index is built from.
///
/// 把文本切分为构建搜索索引所用的词元。
pub fn tokenise(text: &str) -> Vec<String> {
    crate::docs::document::tokenise(text)
}

/// Returns the response a request error becomes.
///
/// 返回某个请求错误所变成的响应。
pub fn error_response(error: RequestError) -> Response {
    crate::http::server::error_response(error)
}

/// Returns the MathML of a formula, without the wrapper that carries it.
///
/// The two steps are one call here because the token type between them is
/// private to the module that owns it and is of no use to a test.
///
/// 返回公式的 MathML，不含承载它的外层包装。
///
/// 两个步骤在这里合并为一次调用，因为二者之间的 token 类型对其所属模块而言是
/// 私有的，测试也用不上。
pub fn mathml(latex: &str) -> String {
    crate::docs::mathml::to_mathml(&crate::docs::mathml::normalize(latex))
}
/// Builds a JSON object literal.
///
/// The tests read better written this way than spelled out, and the macro is an
/// internal construction helper, so it is named here with the other seams.
///
/// 构造一个 JSON 对象字面量。
///
/// 这样写比逐项展开更易读，而该宏是内部的构造辅助，因此与其他接缝一同列在此处。
pub use crate::json_object;
