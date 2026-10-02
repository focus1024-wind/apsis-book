//! The document model, the navigation tree, and the search index.
//!
//! A concept document is an embedded Markdown file with a small metadata
//! header. The header is delimited by lines of three dashes and holds
//! `key: value` pairs; the remainder of the file is the Markdown body.
//!
//! Parsing happens once at startup. Each document keeps both its Markdown
//! source and its rendered HTML, so the service can serve either without
//! rendering again. A document also records the concept identifiers it
//! references, which the service uses to report broken cross references.
//!
//! The chapter list and the order of documents inside a chapter come from
//! `content/concepts/chapters.json`, so the navigation order is authored
//! rather than derived from file names.
//!
//! 文档模型、导航树与搜索索引。
//!
//! 一篇概念文档是一个内嵌的 Markdown 文件，带一小段元数据头部。头部由三短横线的
//! 若干行界定，内含 `key: value` 对；文件的其余部分是 Markdown 正文。
//!
//! 解析在启动时进行一次。每篇文档同时保留其 Markdown 源码与渲染后的 HTML，使
//! 服务无需再次渲染即可提供任一种。文档还记录它所引用的概念标识，服务用它报告
//! 断掉的交叉引用。
//!
//! 章的清单以及章内文档的顺序来自 `content/concepts/chapters.json`，因此导航顺序
//! 是撰写出来的，而不是从文件名推导的。

use std::collections::HashMap;

use crate::docs::markdown;
use crate::json::{self, Value};

/// The language a document variant is written in.
///
/// 一篇文档某一形式所用的语言。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Language {
    /// Simplified Chinese, the default of the interface.
    ///
    /// 简体中文，界面的默认语言。
    Chinese,
    /// English.
    ///
    /// 英文。
    English,
}

impl Language {
    /// Returns the language of a file name suffix.
    ///
    /// A file whose stem ends in `.zh` is Chinese; every other file is
    /// English.
    ///
    /// 按文件名后缀返回语言。
    ///
    /// 主干以 `.zh` 结尾的文件是中文；其他所有文件是英文。
    pub fn of_file(stem: &str) -> Self {
        if stem.ends_with(".zh") {
            Self::Chinese
        } else {
            Self::English
        }
    }

    /// Returns the short tag used in a payload.
    ///
    /// 返回载荷中使用的短标记。
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Chinese => "zh",
            Self::English => "en",
        }
    }
}

/// One language form of a concept document.
///
/// 一篇概念文档的某一种语言形式。
#[derive(Debug, Clone)]
pub struct DocumentVariant {
    /// Display title.
    ///
    /// 显示标题。
    ///
    /// 显示标题。
    ///
    /// 显示标题。
    ///
    /// 显示标题。
    pub title: String,
    /// One sentence summary, shown in listings and search results.
    ///
    /// 一句话概要，显示在列表与搜索结果中。
    pub summary: String,
    /// Markdown source.
    ///
    /// Markdown 源码。
    pub markdown: String,
    /// Rendered HTML.
    ///
    /// 渲染后的 HTML。
    pub html: String,
    /// Section headings, in order.
    ///
    /// 各节标题，按顺序。
    pub headings: Vec<Heading>,
    /// Concept identifiers referenced by the body.
    ///
    /// 正文所引用的概念标识。
    pub references: Vec<String>,
}

/// A parsed concept document, with one form per available language.
///
/// 一篇已解析的概念文档，每种可用语言一种形式。
#[derive(Debug, Clone)]
pub struct Document {
    /// Identifier used in routes and cross references.
    ///
    /// 用于路由与交叉引用的标识。
    pub id: String,
    /// Chapter identifier the document belongs to.
    ///
    /// 文档所属的章标识。
    pub chapter: String,
    /// Free form tags in the language of the English form.
    ///
    /// 自由形式的标签，按英文形式所用的语言。
    pub tags: Vec<String>,
    /// The same tags in Chinese, when the document declares them.
    ///
    /// 同一批标签的中文，若文档声明了它们。
    pub tags_zh: Vec<String>,
    /// Identifier of the document this one builds on, when declared.
    ///
    /// 本篇所依赖的文档标识，若已声明。
    pub prerequisites: Vec<String>,
    /// The English form. Every document has one, and it is the fallback
    /// when a translation is absent.
    ///
    /// 英文形式。每篇文档都有，且在缺少译文时作为回退。
    pub english: DocumentVariant,
    /// The Chinese form, when a translation is embedded.
    ///
    /// 中文形式，若嵌入了译文。
    pub chinese: Option<DocumentVariant>,
}

impl Document {
    /// Parses a document from its embedded text.
    ///
    /// `id` is the identifier the document is addressed by, which comes from
    /// the file name rather than from the header so that a translation can
    /// carry the same identifier as the original. `language` selects the
    /// field the metadata is read into.
    ///
    /// Returns `None` when the mandatory `title` key is absent, because a
    /// document without one cannot be listed.
    ///
    /// 从内嵌文本解析一篇文档。
    ///
    /// `id` 是该文档被寻址所用的标识，它来自文件名而不是头部，使译文能携带与原文
    /// 相同的标识。`language` 选择元数据被读入的字段。
    ///
    /// 必需的 `title` 键缺失时返回 `None`，因为没有标题的文档无法被列出。
    pub fn parse(id: &str, layer: &str, language: Language, text: &str) -> Option<Self> {
        let variant = DocumentVariant::parse(text)?;
        if language == Language::Chinese {
            // A Chinese file on its own is not a document; it is merged with
            // the English one by the loader.

            // 单独一份中文文件不是一篇文档；它由载入器与英文那份合并。
            return None;
        }
        Some(Self {
            id: id.to_string(),
            chapter: layer.to_string(),
            tags: list_of(text, "tags"),
            tags_zh: Vec::new(),
            prerequisites: list_of(text, "prerequisites"),
            english: variant,
            chinese: None,
        })
    }

    /// Returns the form for a language, falling back to English.
    ///
    /// 返回某种语言的形式，回退到英文。
    pub fn variant(&self, language: Language) -> &DocumentVariant {
        match language {
            Language::Chinese => self.chinese.as_ref().unwrap_or(&self.english),
            Language::English => &self.english,
        }
    }

    /// Returns `true` when the document carries a translation.
    ///
    /// 当文档带有译文时返回 `true`。
    pub fn has(&self, language: Language) -> bool {
        match language {
            Language::Chinese => self.chinese.is_some(),
            Language::English => true,
        }
    }

    /// Returns the title in the given language, falling back to English.
    ///
    /// 以给定语言返回标题，回退到英文。
    pub fn title(&self, language: Language) -> &str {
        &self.variant(language).title
    }

    /// Returns the summary in the given language, falling back to English.
    ///
    /// 以给定语言返回概要，回退到英文。
    pub fn summary(&self, language: Language) -> &str {
        &self.variant(language).summary
    }

    /// Returns the plain text of a language form, used for search.
    ///
    /// 返回某个语言形式的纯文本，用于搜索。
    pub fn plain_text(&self, language: Language) -> String {
        markdown::to_plain_text(&self.variant(language).markdown)
    }

    /// Returns every concept identifier the document references, from
    /// either language form.
    ///
    /// 返回文档所引用的每一个概念标识，取自任一语言形式。
    pub fn references(&self) -> Vec<String> {
        let mut references = self.english.references.clone();
        if let Some(chinese) = &self.chinese {
            for reference in &chinese.references {
                if !references.contains(reference) {
                    references.push(reference.clone());
                }
            }
        }
        references
    }
}

impl DocumentVariant {
    /// Parses one language form of a document.
    ///
    /// Returns `None` when the `title` key is absent, because a form without
    /// a title cannot be listed.
    ///
    /// A leading top level heading is removed when it repeats the title. The
    /// page already shows the title, from the metadata, in the heading of the
    /// article, so a body that opens with the same heading would show it
    /// twice. A document whose first heading is a different string keeps it,
    /// because then the two headings say different things.
    ///
    /// 解析文档的某一种语言形式。
    ///
    /// `title` 键缺失时返回 `None`，因为没有标题的形式无法被列出。
    ///
    /// 开头的顶层标题若重复标题本身则被移除。页面已经在文章标题处显示了来自元数据的
    /// 标题，因此以同一标题开头的正文会把它显示两次。首个标题是另一串文字时文档保留
    /// 它，因为那时两个标题说的是不同的事。
    pub fn parse(text: &str) -> Option<Self> {
        let (header, body) = split_header(text);
        let title = field_of(header, "title").unwrap_or_default();
        if title.is_empty() {
            return None;
        }
        let body = strip_leading_title(body, &title);
        Some(Self {
            title,
            summary: field_of(header, "summary").unwrap_or_default(),
            headings: collect_headings(&body),
            references: collect_references(&body),
            markdown: body.clone(),
            html: markdown::render(&body),
        })
    }
}

/// Removes a leading `# heading` when it equals `title`.
///
/// Returns the body unchanged when it does not open with such a heading.
///
/// 当开头的 `# heading` 等于 `title` 时移除它。
///
/// 正文不以这样的标题开头时原样返回。
fn strip_leading_title(body: &str, title: &str) -> String {
    let mut lines = body.lines().peekable();
    // Skip leading blank lines.

    // 跳过开头的空行。
    while matches!(lines.peek(), Some(line) if line.trim().is_empty()) {
        lines.next();
    }
    let Some(first) = lines.peek() else {
        return body.to_string();
    };
    let trimmed = first.trim_end();
    let hashes = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if hashes == 1 && trimmed[hashes..].starts_with(' ') {
        let heading = trimmed[hashes..].trim();
        if heading == title.trim() {
            lines.next();
            // Drop one following blank line so the body does not start with
            // a gap.

            // 去掉随后的一个空行，使正文不以一段空隙开头。
            if matches!(lines.peek(), Some(line) if line.trim().is_empty()) {
                lines.next();
            }
            return lines.collect::<Vec<&str>>().join("\n");
        }
    }
    body.to_string()
}

/// Returns the tag list a document header declares.
///
/// Each language file declares its own tags under the same key, so the
/// loader reads them from the translation rather than from the original.
///
/// 返回文档头部所声明的标签列表。
///
/// 每个语言文件在同一个键下声明自己的标签，因此载入器从译文读取它们而不是从原文。
pub fn tags_of(text: &str) -> Vec<String> {
    list_of(text, "tags")
}

/// Returns the value of a metadata key, trimmed.
///
/// 返回某个元数据键的取值，已去除首尾空白。
fn field_of(header: &str, key: &str) -> Option<String> {
    for line in header.lines() {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case(key) {
            return Some(value.trim().to_string());
        }
    }
    None
}

/// Returns a metadata list value, or an empty list when it is absent.
///
/// 返回某个元数据列表取值，缺失时返回空列表。
fn list_of(text: &str, key: &str) -> Vec<String> {
    let (header, _) = split_header(text);
    field_of(header, key)
        .map(|value| split_list(&value))
        .unwrap_or_default()
}

/// Returns the layer a document belongs to, from the path it was loaded
/// from.
///
/// The hierarchy follows the layers of the library, so a document is placed
/// in the directory named for its layer and the loader reads the layer from
/// there. A file that sits directly in the concept directory belongs to the
/// introduction.
///
/// 从载入路径返回文档所属的层。
///
/// 层级沿用库的层，因此文档被放在以其层命名的目录中，载入器从那里读出层。直接
/// 位于概念目录下的文件属于导论。
pub fn layer_of(relative: &str) -> String {
    match relative.split_once('/') {
        Some((layer, _)) => layer.to_string(),
        None => "overview".to_string(),
    }
}

/// Splits a document into its metadata header and its body.
///
/// 把一篇文档拆分为元数据头部与正文。
fn split_header(text: &str) -> (&str, &str) {
    let trimmed = text.trim_start_matches('\u{feff}');
    let Some(rest) = trimmed.strip_prefix("---") else {
        return ("", trimmed);
    };
    let Some(end) = rest.find("\n---") else {
        return ("", trimmed);
    };
    let header = &rest[..end];
    let body = rest[end + 4..].trim_start_matches(['\n', '\r']);
    (header, body)
}

/// Splits a comma separated list.
///
/// 拆分一个以逗号分隔的列表。
fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn collect_headings(markdown: &str) -> Vec<Heading> {
    let mut headings: Vec<Heading> = Vec::new();
    let mut in_code = false;
    for line in markdown.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        let hashes = trimmed
            .chars()
            .take_while(|character| *character == '#')
            .count();
        if (1..=6).contains(&hashes) && trimmed[hashes..].starts_with(' ') {
            let text = trimmed[hashes..].trim();
            // The anchor must match the one the renderer emits, and two
            // headings can reduce to the same text, so a repeat is numbered
            // the way the renderer numbers it.

            // 锚点必须与渲染器输出的那个一致，而两个标题可能归约为同一文本，
            // 因此重复项按渲染器编号的方式编号。
            let base = slug(text);
            let repeats = headings
                .iter()
                .filter(|heading| heading.base == base)
                .count();
            let anchor = if repeats == 0 {
                base.clone()
            } else {
                format!("{base}-{}", repeats + 1)
            };
            headings.push(Heading {
                base,
                anchor,
                text: text.to_string(),
                level: hashes as u32,
            });
        }
    }
    headings
}

/// One heading of a document.
///
/// 文档的一个标题。
#[derive(Debug, Clone)]
pub struct Heading {
    /// The anchor before the uniqueness suffix, used to detect a repeat.
    ///
    /// 加唯一性后缀之前的锚点，用于检测重复。
    pub base: String,
    /// The anchor the document carries for this heading.
    ///
    /// 文档为该标题所携带的锚点。
    pub anchor: String,
    /// The heading text.
    ///
    /// 标题文本。
    pub text: String,
    /// The heading level, from one to six.
    ///
    /// 标题层级，一到六。
    pub level: u32,
}

/// Returns a URL friendly slug, matching the anchors the Markdown renderer
/// emits.
///
/// 返回 URL 友好的短名，与 Markdown 渲染器输出的锚点一致。
fn slug(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut last_dash = true;
    for character in text.chars() {
        if character.is_alphanumeric() {
            for lowered in character.to_lowercase() {
                slug.push(lowered);
            }
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        slug.push_str("section");
    }
    slug
}

/// Collects the concept identifiers referenced by a Markdown body.
///
/// 收集某个 Markdown 正文所引用的概念标识。
fn collect_references(markdown: &str) -> Vec<String> {
    let mut references = Vec::new();
    let characters: Vec<char> = markdown.chars().collect();
    let mut index = 0;
    while index + 1 < characters.len() {
        if characters[index] == '[' && characters[index + 1] == '[' {
            if let Some(end) = (index + 2..characters.len().saturating_sub(1))
                .find(|position| characters[*position] == ']' && characters[*position + 1] == ']')
            {
                let inner: String = characters[index + 2..end].iter().collect();
                let id = inner
                    .split_once('|')
                    .map(|(id, _)| id)
                    .unwrap_or(&inner)
                    .trim();
                if !id.is_empty() && !references.iter().any(|existing| existing == id) {
                    references.push(id.to_string());
                }
                index = end + 2;
                continue;
            }
        }
        index += 1;
    }
    references
}

/// A top level group of the documentation.
///
/// The documentation is presented as two groups: the concepts, which describe
/// the field, and the implementation, which describes what a particular
/// library does. The concepts are the larger superset; the implementation
/// exists so that a reader who wants to know what the code does can find it
/// without it being mixed into the teaching material.
///
/// 文档的一个顶层分组。
///
/// 文档以两个分组呈现：概念，描述这个领域；实现，描述某一个特定库做了什么。
/// 概念是更大的超集；实现之所以存在，是为了让想知道代码做什么的读者能找到它，
/// 而不必把它混进教学材料里。
#[derive(Debug, Clone)]
pub struct Group {
    /// Group identifier.
    ///
    /// 分组标识。
    pub id: String,
    /// Display title.
    pub title: String,
    /// Display title in Chinese, when the chapter list gives one.
    ///
    /// 中文显示标题，若章清单给出了一个。
    ///
    /// 中文显示标题，若章清单给出了一个。
    ///
    /// 中文显示标题，若章清单给出了一个。
    pub title_zh: Option<String>,
    /// The layers or sections of the group, in reading order.
    ///
    /// 该分组的各层或各节，按阅读顺序。
    pub sections: Vec<Chapter>,
}

/// A chapter, or layer, of a group.
///
/// A layer that covers two unrelated subjects declares a subsection for each,
/// so that a reader is not shown one flat list of documents that have
/// nothing to do with one another. When the list is empty the layer holds its
/// documents directly.
///
/// 分组的一个章，也就是一个层。
///
/// 涵盖两个无关主题的层为每个主题声明一个子区，使读者不会看到一份把互不相干的
/// 文档凑在一起的平铺列表。该列表为空时，层直接持有它的文档。
#[derive(Debug, Clone)]
pub struct Chapter {
    /// The subjects of the layer, when it is presented as more than one.
    ///
    /// Empty when the layer holds its documents directly.
    ///
    /// 该层的各主题，当它被呈现为多于一个时。
    ///
    /// 层直接持有其文档时为空。
    pub subsections: Vec<Section>,
    /// Chapter identifier.
    ///
    /// 章标识。
    pub id: String,
    /// Display title.
    pub title: String,
    /// Display title in Chinese, when the chapter list gives one.
    pub title_zh: Option<String>,
    /// One sentence description.
    ///
    /// 一句话描述。
    ///
    /// 一句话描述。
    pub summary: String,
    /// One sentence description in Chinese, when the chapter list gives one.
    ///
    /// 一句话中文描述，若章清单给出了一个。
    pub summary_zh: Option<String>,
    /// Document identifiers in reading order.
    ///
    /// 文档标识，按阅读顺序。
    pub documents: Vec<String>,
}

/// One subject within a layer, used when a layer covers two that are
/// unrelated.
///
/// 层内的一个主题，用于层涵盖两个彼此无关主题的情形。
#[derive(Debug, Clone)]
pub struct Section {
    /// Section identifier.
    ///
    /// 子区标识。
    pub id: String,
    /// Display title.
    pub title: String,
    /// Display title in Chinese, when the chapter list gives one.
    pub title_zh: Option<String>,
    /// One sentence description.
    pub summary: String,
    /// The same description in Chinese, when the chapter list gives one.
    ///
    /// 同一描述的中文，若章清单给出了一个。
    pub summary_zh: Option<String>,
    /// Document identifiers, in reading order.
    ///
    /// 文档标识，按阅读顺序。
    pub documents: Vec<String>,
}

/// The document collection plus its navigation order.
///
/// 文档集合及其导航顺序。
#[derive(Debug)]
pub struct Tree {
    documents: HashMap<String, Document>,
    groups: Vec<Group>,
    order: Vec<String>,
}

impl Tree {
    /// Returns the reading order, the documents in the order they are meant
    /// to be read.
    ///
    /// This is the order the frontend navigates and the search index weights,
    /// so a caller that lists the documents needs it.
    ///
    /// 返回阅读顺序，即各文档按其被阅读的次序排列。
    ///
    /// 这是前端导航与搜索索引加权所用的顺序，因此列出文档的调用方需要它。
    pub fn reading_order(&self) -> &[String] {
        &self.order
    }

    /// Builds the tree from the embedded content.
    ///
    /// 从内嵌内容构建该树。
    pub fn new(documents: Vec<Document>) -> Self {
        let declared = load_groups();
        let mut map: HashMap<String, Document> = HashMap::new();
        for document in documents {
            map.insert(document.id.clone(), document);
        }

        // A document whose layer is not declared in the chapter list is
        // appended to the group that matches, creating the layer when it is
        // missing, so that a new file is visible before the list is
        // updated rather than invisible.

        // 层未在章清单中声明的文档被追加到相匹配的分组，并在该层缺失时创建它，
        // 使新文件在清单更新之前就可见，而不是不可见。
        let mut groups = declared;
        let mut listed: Vec<String> = groups
            .iter()
            .flat_map(|group| group.sections.iter())
            .flat_map(|section| section.documents.iter().cloned())
            .collect();
        listed.sort();
        let unlisted: Vec<String> = map
            .keys()
            .filter(|id| listed.binary_search(id).is_err())
            .cloned()
            .collect();

        for id in unlisted {
            let layer = map
                .get(&id)
                .map(|document| document.chapter.clone())
                .unwrap_or_else(|| "overview".to_string());
            let holder = groups
                .iter_mut()
                .flat_map(|group| group.sections.iter_mut())
                .find(|section| section.id == layer);
            match holder {
                Some(section) if section.subsections.is_empty() => section.documents.push(id),
                Some(section) => {
                    // The layer is presented as subjects, so a document that
                    // the chapter list already placed in one of them stays
                    // where it is. Only an unlisted document is appended,
                    // and it goes into the first subject rather than beside
                    // them.

                    // 该层以主题呈现，因此章清单已放入某个主题的文档留在原处。
                    // 只追加未列出的文档，且放入第一个主题，而不是与各主题并列。
                    let already_listed = section
                        .subsections
                        .iter()
                        .any(|subject| subject.documents.contains(&id));
                    if !already_listed {
                        section
                            .subsections
                            .first_mut()
                            .expect("the layer has subjects")
                            .documents
                            .push(id);
                    }
                }
                None => {
                    let section = Chapter {
                        id: layer.clone(),
                        title: title_case(&layer),
                        title_zh: None,
                        summary: String::new(),
                        summary_zh: None,
                        documents: vec![id],
                        subsections: Vec::new(),
                    };
                    match groups.iter_mut().find(|group| group.id == "implementation") {
                        Some(group) => group.sections.push(section),
                        None => groups.push(Group {
                            id: "implementation".to_string(),
                            title: "Implementation".to_string(),
                            title_zh: None,
                            sections: vec![section],
                        }),
                    }
                }
            }
        }

        // A section may name a document that does not exist; drop it so that
        // navigation never advertises a missing page.

        // 章可能指名一个不存在的文档；丢弃它，使导航绝不宣传一个缺失的页面。
        for group in &mut groups {
            for section in &mut group.sections {
                section.documents.retain(|id| map.contains_key(id));
                for subject in &mut section.subsections {
                    subject.documents.retain(|id| map.contains_key(id));
                }
                section
                    .subsections
                    .retain(|subject| !subject.documents.is_empty());
                // A layer that declares subjects holds its documents in
                // them. The layer's own list is dropped, because a document
                // named in both places would be offered twice and the two
                // entries would share a key, which stops the page rendering.

                // 声明了主题的层把文档放在主题里。层自己的列表被丢弃，因为两处
                // 都指名的文档会被提供两次，而两个条目会共用一个键，那会使页面
                // 停止渲染。
                if !section.subsections.is_empty() {
                    section.documents.clear();
                }
            }
            group
                .sections
                .retain(|section| !section.documents.is_empty() || !section.subsections.is_empty());
        }
        groups.retain(|group| !group.sections.is_empty());

        let order: Vec<String> = groups
            .iter()
            .flat_map(|group| group.sections.iter())
            .flat_map(|section| {
                section.documents.iter().cloned().chain(
                    section
                        .subsections
                        .iter()
                        .flat_map(|subject| subject.documents.iter().cloned()),
                )
            })
            .collect();

        Self {
            documents: map,
            groups,
            order,
        }
    }

    /// Returns a document by identifier.
    ///
    /// 按标识返回一篇文档。
    pub fn document(&self, id: &str) -> Option<&Document> {
        self.documents.get(id)
    }

    /// Returns the documents in navigation order.
    ///
    /// 按导航顺序返回各文档。
    pub fn ordered(&self) -> impl Iterator<Item = &Document> {
        self.order.iter().filter_map(|id| self.documents.get(id))
    }

    /// Returns the groups, each holding its layers.
    ///
    /// 返回各分组，每个分组持有自己的各层。
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }

    /// Returns the number of documents.
    ///
    /// 返回文档数量。
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Returns `true` when there are no documents.
    ///
    /// 没有文档时返回 `true`。
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Returns every cross reference that does not resolve to a document.
    ///
    /// The result is a list of pairs of the referring document identifier
    /// and the missing identifier. The check runs over the embedded content
    /// when the service starts, so a mistyped cross reference is caught
    /// rather than becoming a dead link in a page.
    ///
    /// 返回每一个未能解析到文档的交叉引用。
    ///
    /// 结果是一组「引用方文档标识」与「缺失标识」构成的二元组。该检查在服务启动时
    /// 遍历内嵌内容，因此写错的交叉引用会被抓住，而不是变成页面中的死链。
    #[allow(dead_code)]
    pub fn missing_links(&self) -> Vec<(String, String)> {
        let mut missing = Vec::new();
        for document in self.ordered() {
            for reference in document.references() {
                if !self.documents.contains_key(&reference) {
                    missing.push((document.id.clone(), reference));
                }
            }
        }
        missing
    }

    /// Builds the search index over the collection.
    ///
    /// Each language form is indexed separately, so a query in one language
    /// ranks the documents that contain it in that language. A document
    /// without a translation contributes only its English form.
    ///
    /// 在集合之上构建搜索索引。
    ///
    /// 每种语言形式单独建索引，因此某种语言的查询会给该语言中含有它的文档打分。
    /// 没有译文的文档只贡献其英文形式。
    pub fn search_index(&self) -> SearchIndex {
        let mut entries = Vec::new();
        for document in self.ordered() {
            for language in [Language::Chinese, Language::English] {
                if !document.has(language) {
                    continue;
                }
                entries.push(IndexEntry {
                    id: document.id.clone(),
                    language,
                    title: document.title(language).to_string(),
                    summary: document.summary(language).to_string(),
                    chapter: document.chapter.clone(),
                    text: document.plain_text(language).to_lowercase(),
                });
            }
        }
        SearchIndex::build(entries)
    }
}

/// Turns `orbit-model` into `Orbit model`.
///
/// 把 `orbit-model` 变成 `Orbit model`。
fn title_case(id: &str) -> String {
    let mut text = id.replace(['-', '_'], " ");
    if let Some(first) = text.chars().next() {
        text.replace_range(0..first.len_utf8(), &first.to_uppercase().to_string());
    }
    text
}

/// Loads the groups and their layers from `content/concepts/chapters.json`.
///
/// The file is the single place that says what exists and in what order. The
/// directory tree says which layer a document belongs to, so the two are
/// cross-checked rather than repeated: a document listed under one layer and
/// stored in another is a mistake the reader would not see.
///
/// 从 `content/concepts/chapters.json` 载入各分组及其各层。
///
/// 该文件是「存在什么、以何顺序」的唯一出处。目录树说明一篇文档属于哪一层，
/// 因此两者互相校验而不是重复：列在一个层下却存放在另一个层，是读者看不到的
/// 错误。
fn load_groups() -> Vec<Group> {
    let Some(text) = crate::docs::content::get("concepts/chapters.json") else {
        return Vec::new();
    };
    let Ok(value) = json::parse(text) else {
        return Vec::new();
    };
    value
        .get("groups")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(parse_group)
        .collect()
}

/// Parses one group and its layers.
///
/// 解析一个分组及其各层。
fn parse_group(value: &Value) -> Option<Group> {
    let id = value.get("id")?.as_str()?.to_string();
    let sections = value
        .get("sections")
        .map(Value::as_array)
        .unwrap_or_default()
        .iter()
        .filter_map(parse_section)
        .collect();
    Some(Group {
        id: id.clone(),
        title: value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(&id)
            .to_string(),
        title_zh: value
            .get("title_zh")
            .and_then(Value::as_str)
            .map(str::to_string),
        sections,
    })
}

/// Parses one layer of a group.
///
/// 解析分组的一个层。
fn parse_section(value: &Value) -> Option<Chapter> {
    let id = value.get("id")?.as_str()?.to_string();
    Some(Chapter {
        title: value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(&id)
            .to_string(),
        title_zh: value
            .get("title_zh")
            .and_then(Value::as_str)
            .map(str::to_string),
        summary: value
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        summary_zh: value
            .get("summary_zh")
            .and_then(Value::as_str)
            .map(str::to_string),
        documents: value
            .get("documents")
            .map(Value::as_array)
            .unwrap_or_default()
            .iter()
            .filter_map(|document| document.as_str().map(str::to_string))
            .collect(),
        subsections: value
            .get("subsections")
            .map(Value::as_array)
            .unwrap_or_default()
            .iter()
            .filter_map(parse_subsection)
            .collect(),
        id,
    })
}

/// Parses one subject within a layer.
///
/// 解析层内的一个主题。
fn parse_subsection(value: &Value) -> Option<Section> {
    let id = value.get("id")?.as_str()?.to_string();
    Some(Section {
        title: value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(&id)
            .to_string(),
        title_zh: value
            .get("title_zh")
            .and_then(Value::as_str)
            .map(str::to_string),
        summary: value
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        summary_zh: value
            .get("summary_zh")
            .and_then(Value::as_str)
            .map(str::to_string),
        documents: value
            .get("documents")
            .map(Value::as_array)
            .unwrap_or_default()
            .iter()
            .filter_map(|document| document.as_str().map(str::to_string))
            .collect(),
        id,
    })
}

/// One document in the search index.
///
/// 搜索索引中的一篇文档。
struct IndexEntry {
    id: String,
    language: Language,
    title: String,
    summary: String,
    chapter: String,
    text: String,
}

/// A search index over the documents.
///
/// The index maps a lowercased token to the documents that contain it. A
/// query is tokenised the same way and scored by the number of distinct
/// query tokens a document matches, with a bonus for a token that appears in
/// the title. The result is small enough that a linear scan over the
/// postings is faster than any more elaborate structure.
///
/// 覆盖各文档的搜索索引。
///
/// 索引把一个小写的词元映射到包含它的文档。查询以同样方式分词，并按一篇文档匹配
/// 到的查询词元个数打分，出现在标题中的词元另有加分。结果规模很小，对倒排表做
/// 线性扫描比任何更精巧的结构都快。
pub struct SearchIndex {
    entries: Vec<SearchEntry>,
    postings: HashMap<String, Vec<(usize, u32)>>,
}

/// A document in the index, with the fields shown in a result.
///
/// 索引中的一篇文档，含结果中显示的各个字段。
struct SearchEntry {
    id: String,
    language: Language,
    title: String,
    summary: String,
    chapter: String,
    title_tokens: Vec<String>,
    /// The searchable text, lowercased, kept for the substring fallback that
    /// a query of a single character uses.
    ///
    /// 可搜索的文本，已转为小写，保留给单字符查询所用的子串回退。
    haystack: String,
}

impl SearchIndex {
    /// Builds an index from the entries.
    ///
    /// 由各条目构建索引。
    fn build(entries: Vec<IndexEntry>) -> Self {
        let mut postings: HashMap<String, Vec<(usize, u32)>> = HashMap::new();
        let mut search_entries = Vec::with_capacity(entries.len());

        for (index, entry) in entries.iter().enumerate() {
            let title_tokens = tokenise(&entry.title.to_lowercase());
            for token in tokenise(&entry.text) {
                let counts = postings.entry(token).or_default();
                match counts.last_mut() {
                    Some((last, count)) if *last == index => *count += 1,
                    _ => counts.push((index, 1)),
                }
            }
            for token in &title_tokens {
                let counts = postings.entry(token.clone()).or_default();
                if !counts.iter().any(|(entry, _)| *entry == index) {
                    counts.push((index, 0));
                }
            }
            search_entries.push(SearchEntry {
                id: entry.id.clone(),
                language: entry.language,
                title: entry.title.clone(),
                summary: entry.summary.clone(),
                chapter: entry.chapter.clone(),
                title_tokens,
                haystack: format!("{} {} {}", entry.title, entry.summary, entry.text)
                    .to_lowercase(),
            });
        }

        Self {
            entries: search_entries,
            postings,
        }
    }

    /// Searches the index and returns the best matches, ordered by score.
    ///
    /// At most `limit` results are returned. A query with no usable token and
    /// no literal match returns nothing.
    ///
    /// A query is handled in two ways. A query that yields tokens is matched
    /// against the postings. A query that yields no token, such as a single
    /// character, is matched as a substring, because a one character query
    /// cannot form a token and would otherwise always return nothing.
    ///
    /// 搜索索引并返回最佳匹配，按得分排序。
    ///
    /// 至多返回 `limit` 条结果。既无可用词元又无字面匹配的查询返回空。
    ///
    /// 查询分两种方式处理。能产出词元的查询与倒排表匹配。产不出词元的查询——例如
    /// 单个字符——按子串匹配，因为单字符查询无法构成词元，否则将永远一无所获。
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }
        let tokens = tokenise(&needle);
        if tokens.is_empty() {
            return self.search_substring(&needle, limit);
        }

        let mut scores: HashMap<usize, f64> = HashMap::new();
        for token in &tokens {
            let Some(postings) = self.postings.get(token) else {
                continue;
            };
            // A token that appears in many documents is less informative.

            // 出现在许多文档中的词元信息量更小。
            let weight = 1.0 / (1.0 + (postings.len() as f64).ln());
            for (index, count) in postings {
                let entry = &self.entries[*index];
                let title_bonus = if entry.title_tokens.contains(token) {
                    3.0
                } else {
                    0.0
                };
                let frequency = 1.0 + (*count as f64).sqrt();
                *scores.entry(*index).or_insert(0.0) += weight * (frequency + title_bonus);
            }
        }

        let mut hits: Vec<SearchHit> = scores
            .into_iter()
            .map(|(index, score)| {
                let entry = &self.entries[index];
                SearchHit {
                    id: entry.id.clone(),
                    language: entry.language,
                    title: entry.title.clone(),
                    summary: entry.summary.clone(),
                    chapter: entry.chapter.clone(),
                    score,
                }
            })
            .collect();
        hits.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.id.cmp(&right.id))
        });
        hits.truncate(limit);
        hits
    }

    /// Returns the documents whose text contains the query literally.
    ///
    /// Used for a query that yields no token. A match in the title scores
    /// higher than a match in the body, so the ordering stays useful.
    ///
    /// 返回文本字面包含该查询的文档。
    ///
    /// 用于产不出词元的查询。标题中的匹配得分高于正文中的匹配，使排序仍然有用。
    fn search_substring(&self, needle: &str, limit: usize) -> Vec<SearchHit> {
        let mut hits: Vec<SearchHit> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.haystack.contains(needle))
            .map(|(index, entry)| {
                let title_match = entry.title.to_lowercase().contains(needle);
                SearchHit {
                    id: entry.id.clone(),
                    language: entry.language,
                    title: entry.title.clone(),
                    summary: entry.summary.clone(),
                    chapter: entry.chapter.clone(),
                    score: if title_match { 2.0 } else { 1.0 } + (index as f64) * 1e-9,
                }
            })
            .collect();
        hits.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.id.cmp(&right.id))
        });
        hits.dedup_by(|left, right| left.id == right.id);
        hits.truncate(limit);
        hits
    }

    /// Returns the number of indexed documents.
    ///
    /// 返回已建索引的文档数量。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when the index is empty.
    ///
    /// 索引为空时返回 `true`。
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// One search result.
///
/// 一条搜索结果。
#[derive(Debug, Clone)]
pub struct SearchHit {
    /// Identifier of the matching document.
    ///
    /// 匹配文档的标识。
    pub id: String,
    /// Language form that matched.
    ///
    /// 匹配到的语言形式。
    pub language: Language,
    /// Title of the matching document.
    ///
    /// 匹配文档的标题。
    pub title: String,
    /// Summary of the matching document.
    ///
    /// 匹配文档的概要。
    pub summary: String,
    /// Chapter the document belongs to.
    ///
    /// 文档所属的章。
    pub chapter: String,
    /// Relevance score.
    ///
    /// 相关性得分。
    pub score: f64,
}

/// Splits text into lowercased tokens.
///
/// Two kinds of token are produced:
///
/// - A run of letters and digits becomes one token per word, with identifiers
///   written with underscores, hyphens, or dots split there as well, so
///   `mean_motion` and `mean-motion` both match a query for `mean motion`.
///   A word of one character is dropped, because it would match too much.
/// - A run of Chinese characters becomes one token per adjacent pair. Chinese
///   text is not written with spaces, so a word cannot be recovered by
///   splitting on whitespace; a pair of adjacent characters is the smallest
///   unit that is specific enough to be a word. Searching 坐标 therefore
///   matches a paragraph that contains 坐标, which a whole-run token would
///   not, because the run around it is longer.
///
/// 把文本切分为小写的词元。
///
/// 产生两类词元：
///
/// - 一串字母与数字按词产生一个词元，用下划线、连字符或点书写的标识也在那里
///   拆开，因此 `mean_motion` 与 `mean-motion` 都能匹配对 `mean motion` 的
///   查询。单个字符的词被丢弃，因为它会匹配太多。
/// - 一串中文字符每相邻两个产生一个词元。中文书写不带空白，因此无法靠按空白
///   切分恢复出词；相邻的一对字符是仍足够具体到能视作一个词的最小单位。查询
///   坐标因此能匹配含有 坐标 的段落，而整串词元做不到，因为它周围的串更长。
pub(crate) fn tokenise(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut han: Vec<char> = Vec::new();

    // Flushes the pending Latin word.

    // 冲刷尚未结算的拉丁词。
    fn flush_word(word: &mut String, tokens: &mut Vec<String>) {
        if word.chars().count() >= 2 {
            tokens.push(word.clone());
        }
        word.clear();
    }

    // Flushes the pending run of Chinese characters as adjacent pairs. A
    // lone character produces no token; a query of one character is handled
    // by the substring search instead, which is more precise than a token
    // that would match every occurrence of that character.

    // 把尚未结算的一串中文字符按相邻对冲刷。单个字符不产生词元；单字符查询改由
    // 子串搜索处理，那比一个会匹配该字符每一处出现的词元更精确。
    fn flush_han(han: &mut Vec<char>, tokens: &mut Vec<String>) {
        for pair in han.windows(2) {
            tokens.push(pair.iter().collect());
        }
        han.clear();
    }

    for character in text.chars() {
        if is_han(character) {
            flush_word(&mut word, &mut tokens);
            han.push(character);
        } else if character.is_alphanumeric() {
            flush_han(&mut han, &mut tokens);
            word.extend(character.to_lowercase());
        } else {
            flush_word(&mut word, &mut tokens);
            flush_han(&mut han, &mut tokens);
        }
    }
    flush_word(&mut word, &mut tokens);
    flush_han(&mut han, &mut tokens);
    tokens
}

/// Returns `true` when a character is a Chinese ideograph.
///
/// The ranges cover the unified ideographs, the extension A block, and the
/// compatibility ideographs, which is everything the content uses.
///
/// 当某个字符是汉字时返回 `true`。
///
/// 各范围覆盖统一表意文字、扩展 A 区与兼容表意文字，即内容所用的全部字符。
fn is_han(character: char) -> bool {
    matches!(character as u32,
        0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff)
}
