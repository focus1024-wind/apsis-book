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
//!
//! 概念文档，以及从 `apsis-rs` 提取的接口目录。
//!
//! 服务关于文档所知的一切都归本模块所有：
//!
//! - [`content`] 使内嵌的 `content/` 文件可用。
//! - [`markdown`] 把文档正文转为 HTML。
//! - [`document`] 为文档、章节树与搜索索引建模。
//! - [`api`] 为从 `apsis-rs` 源码提取的接口目录建模。
//!
//! [`load_documents`] 与 [`load_api_catalog`] 在启动时从内嵌内容组装这两个集合。
//!
//! 一篇文档以英文写一次，为 `concepts/<id>.md`。中文译文是
//! `concepts/<id>.zh.md`，其元数据与正文对语言为中文的读者取代英文部分。没有译文
//! 的文档对所有读者都以英文提供。

pub mod api;
pub mod content;
pub mod document;
pub mod markdown;
pub mod mathml;

#[allow(unused_imports)]
pub use api::ApiItem;
pub use api::{ApiCatalog, ApiCrate};

pub use document::{Document, DocumentVariant, Language, SearchIndex, Tree};

/// Parses every embedded concept document, merging the translations.
///
/// A file that fails to parse is skipped; the service reports the remaining
/// documents and the document route answers with a not found status for the
/// missing one.
///
/// 解析每一篇内嵌概念文档，并合并译文。
///
/// 解析失败的文件被跳过；服务报告其余文档，而对缺失的那一篇，文档路由以「未找到」
/// 状态作答。
pub fn load_documents() -> Vec<Document> {
    // The English files are the base of the collection. The identifier is
    // the file stem with the language suffix removed, so a translation and
    // its original meet on the same key.

    // 英文文件是集合的基础。标识是去掉语言后缀的文件名主干，因此译文与其原文
    // 在同一个键上相遇。
    let mut order: Vec<String> = Vec::new();
    let mut documents: std::collections::HashMap<String, Document> =
        std::collections::HashMap::new();

    for (path, text) in content::list("concepts/", ".md") {
        let Some(relative) = path.strip_prefix("concepts/") else {
            continue;
        };
        // The identifier is the file name; the directory above it is the
        // layer, which is a separate piece of information.

        // 标识即文件名；其上一级目录是层，那是另一份信息。
        let Some(stem) = relative
            .rsplit_once('/')
            .map_or(Some(relative), |(_, name)| Some(name))
        else {
            continue;
        };
        let Some(stem) = stem.strip_suffix(".md") else {
            continue;
        };
        let language = Language::of_file(stem);
        if language != Language::English {
            continue;
        }
        let layer = crate::docs::document::layer_of(relative);
        if let Some(document) = Document::parse(stem, &layer, Language::English, text) {
            order.push(stem.to_string());
            documents.insert(stem.to_string(), document);
        }
    }

    for (path, text) in content::list("concepts/", ".zh.md") {
        let Some(stem) = path
            .strip_prefix("concepts/")
            .and_then(|name| {
                name.rsplit_once('/')
                    .map_or(Some(name), |(_, base)| Some(base))
            })
            .and_then(|name| name.strip_suffix(".zh.md"))
        else {
            continue;
        };
        // A translation without an English original is ignored rather than
        // served on its own, so that the collection always has one entry per
        // identifier.

        // 没有英文原文的译文被忽略，而不单独提供，这样集合中每个标识始终只有
        // 一个条目。
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
///
/// 解析内嵌的接口目录。
pub fn load_api_catalog() -> ApiCatalog {
    let Some(text) = content::get("api.json") else {
        return ApiCatalog {
            crates: Vec::new(),
            generated_from: Vec::new(),
            item_names: std::collections::HashSet::new(),
        };
    };
    api::parse(text)
}
