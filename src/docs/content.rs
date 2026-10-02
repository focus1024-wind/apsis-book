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
//!
//! 对内嵌在二进制中的撰写文本的访问。
//!
//! `build.rs` 扫描 `content/` 目录并生成 `$OUT_DIR/embedded_content.rs`，
//! 其中的 `CONTENT` 表把相对 `content/` 的路径映射到文件文本。文档层消费该表，
//! 因此服务在运行期从不从文件系统读取内容文件。
//!
//! `content/api.json` 下的接口目录由 `scripts/extract_api.sh` 从 `apsis-rs`
//! 源码生成，并经同一张表传递。

include!(concat!(env!("OUT_DIR"), "/embedded_content.rs"));

/// Returns the embedded text for a path relative to `content/`.
///
/// 返回相对 `content/` 的某个路径所对应的内嵌文本。
pub fn get(path: &str) -> Option<&'static str> {
    CONTENT
        .iter()
        .find(|(name, _)| *name == path)
        .map(|(_, text)| *text)
}

/// Returns every embedded entry whose path starts with `prefix` and ends
/// with `suffix`.
///
/// 返回每一个路径以 `prefix` 开头、以 `suffix` 结尾的内嵌条目。
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
///
/// 当至少有一个内容文件被嵌入时返回 `true`。
///
/// 服务在健康响应中报告这一点，因此没有内容的构建会如实说明，而不是提供一个空
/// 文档集。
#[allow(dead_code)]
pub fn is_populated() -> bool {
    !CONTENT.is_empty()
}
