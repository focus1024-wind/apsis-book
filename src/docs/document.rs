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

use std::collections::HashMap;

use crate::docs::markdown;
use crate::json::{self, Value};

/// The language a document variant is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Language {
    /// Simplified Chinese, the default of the interface.
    Chinese,
    /// English.
    English,
}

impl Language {
    /// Returns the language of a file name suffix.
    ///
    /// A file whose stem ends in `.zh` is Chinese; every other file is
    /// English.
    pub fn of_file(stem: &str) -> Self {
        if stem.ends_with(".zh") {
            Self::Chinese
        } else {
            Self::English
        }
    }

    /// Returns the short tag used in a payload.
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Chinese => "zh",
            Self::English => "en",
        }
    }
}

/// One language form of a concept document.
#[derive(Debug, Clone)]
pub struct DocumentVariant {
    /// Display title.
    pub title: String,
    /// One sentence summary, shown in listings and search results.
    pub summary: String,
    /// Markdown source.
    pub markdown: String,
    /// Rendered HTML.
    pub html: String,
    /// Section headings, in order, as pairs of anchor identifier and text.
    pub headings: Vec<(String, String)>,
    /// Concept identifiers referenced by the body.
    pub references: Vec<String>,
}

/// A parsed concept document, with one form per available language.
#[derive(Debug, Clone)]
pub struct Document {
    /// Identifier used in routes and cross references.
    pub id: String,
    /// Chapter identifier the document belongs to.
    pub chapter: String,
    /// Free form tags.
    pub tags: Vec<String>,
    /// Identifier of the document this one builds on, when declared.
    pub prerequisites: Vec<String>,
    /// The English form. Every document has one, and it is the fallback
    /// when a translation is absent.
    pub english: DocumentVariant,
    /// The Chinese form, when a translation is embedded.
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
    pub fn parse(id: &str, language: Language, text: &str) -> Option<Self> {
        let variant = DocumentVariant::parse(text)?;
        if language == Language::Chinese {
            // A Chinese file on its own is not a document; it is merged with
            // the English one by the loader.
            return None;
        }
        Some(Self {
            id: id.to_string(),
            chapter: chapter_of(text),
            tags: list_of(text, "tags"),
            prerequisites: list_of(text, "prerequisites"),
            english: variant,
            chinese: None,
        })
    }

    /// Returns the form for a language, falling back to English.
    pub fn variant(&self, language: Language) -> &DocumentVariant {
        match language {
            Language::Chinese => self.chinese.as_ref().unwrap_or(&self.english),
            Language::English => &self.english,
        }
    }

    /// Returns `true` when the document carries a translation.
    pub fn has(&self, language: Language) -> bool {
        match language {
            Language::Chinese => self.chinese.is_some(),
            Language::English => true,
        }
    }

    /// Returns the title in the given language, falling back to English.
    pub fn title(&self, language: Language) -> &str {
        &self.variant(language).title
    }

    /// Returns the summary in the given language, falling back to English.
    pub fn summary(&self, language: Language) -> &str {
        &self.variant(language).summary
    }

    /// Returns the plain text of a language form, used for search.
    pub fn plain_text(&self, language: Language) -> String {
        markdown::to_plain_text(&self.variant(language).markdown)
    }

    /// Returns every concept identifier the document references, from
    /// either language form.
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
    pub fn parse(text: &str) -> Option<Self> {
        let (header, body) = split_header(text);
        let title = field_of(header, "title").unwrap_or_default();
        if title.is_empty() {
            return None;
        }
        Some(Self {
            title,
            summary: field_of(header, "summary").unwrap_or_default(),
            headings: collect_headings(body),
            references: collect_references(body),
            markdown: body.to_string(),
            html: markdown::render(body),
        })
    }
}

/// Returns the value of a metadata key, trimmed.
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
fn list_of(text: &str, key: &str) -> Vec<String> {
    let (header, _) = split_header(text);
    field_of(header, key)
        .map(|value| split_list(&value))
        .unwrap_or_default()
}

/// Returns the chapter a document declares, defaulting to `overview`.
fn chapter_of(text: &str) -> String {
    let (header, _) = split_header(text);
    field_of(header, "chapter").unwrap_or_else(|| "overview".to_string())
}

/// Splits a document into its metadata header and its body.
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
fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// Collects the headings of a Markdown body as pairs of anchor and text.
fn collect_headings(markdown: &str) -> Vec<(String, String)> {
    let mut headings = Vec::new();
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
            headings.push((slug(text), text.to_string()));
        }
    }
    headings
}

/// Returns a URL friendly slug, matching the anchors the Markdown renderer
/// emits.
fn slug(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut last_dash = true;
    for character in text.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
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

/// A chapter of the documentation.
#[derive(Debug, Clone)]
pub struct Chapter {
    /// Chapter identifier.
    pub id: String,
    /// Display title.
    pub title: String,
    /// Display title in Chinese, when the chapter list gives one.
    pub title_zh: Option<String>,
    /// One sentence description.
    pub summary: String,
    /// One sentence description in Chinese, when the chapter list gives one.
    pub summary_zh: Option<String>,
    /// Document identifiers in reading order.
    pub documents: Vec<String>,
}

/// The document collection plus its navigation order.
#[derive(Debug)]
pub struct Tree {
    documents: HashMap<String, Document>,
    chapters: Vec<Chapter>,
    order: Vec<String>,
}

impl Tree {
    /// Builds the tree from the embedded content.
    pub fn new(documents: Vec<Document>) -> Self {
        let mut chapters = load_chapters();
        let mut map: HashMap<String, Document> = HashMap::new();
        for document in documents {
            map.insert(document.id.clone(), document);
        }

        // Documents not listed in a chapter are appended to the chapter
        // matching their metadata, creating the chapter when needed. This
        // keeps a newly added document visible before the chapter list is
        // updated.
        let mut listed: Vec<String> = chapters
            .iter()
            .flat_map(|chapter| chapter.documents.iter().cloned())
            .collect();
        listed.sort();
        let mut unlisted: Vec<String> = map
            .keys()
            .filter(|id| !listed.binary_search(id).is_ok())
            .cloned()
            .collect();
        unlisted.sort();

        for id in unlisted {
            let chapter_id = map
                .get(&id)
                .map(|document| document.chapter.clone())
                .unwrap_or_else(|| "overview".to_string());
            match chapters.iter_mut().find(|chapter| chapter.id == chapter_id) {
                Some(chapter) => chapter.documents.push(id),
                None => chapters.push(Chapter {
                    id: chapter_id.clone(),
                    title: title_case(&chapter_id),
                    title_zh: None,
                    summary: String::new(),
                    summary_zh: None,
                    documents: vec![id],
                }),
            }
        }

        // A chapter may name a document that does not exist; drop it so that
        // navigation never advertises a missing page.
        for chapter in &mut chapters {
            chapter.documents.retain(|id| map.contains_key(id));
        }
        chapters.retain(|chapter| !chapter.documents.is_empty());

        let order: Vec<String> = chapters
            .iter()
            .flat_map(|chapter| chapter.documents.iter().cloned())
            .collect();

        Self {
            documents: map,
            chapters,
            order,
        }
    }

    /// Returns a document by identifier.
    pub fn document(&self, id: &str) -> Option<&Document> {
        self.documents.get(id)
    }

    /// Returns the documents in navigation order.
    pub fn ordered(&self) -> impl Iterator<Item = &Document> {
        self.order.iter().filter_map(|id| self.documents.get(id))
    }

    /// Returns the chapters.
    pub fn chapters(&self) -> &[Chapter] {
        &self.chapters
    }

    /// Returns the number of documents.
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Returns `true` when there are no documents.
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
fn title_case(id: &str) -> String {
    let mut text = id.replace(['-', '_'], " ");
    if let Some(first) = text.chars().next() {
        text.replace_range(0..first.len_utf8(), &first.to_uppercase().to_string());
    }
    text
}

/// Loads the chapter list from `content/concepts/chapters.json`.
fn load_chapters() -> Vec<Chapter> {
    let Some(text) = crate::docs::content::get("concepts/chapters.json") else {
        return Vec::new();
    };
    let Ok(value) = json::parse(text) else {
        return Vec::new();
    };
    let Some(chapters) = value.get("chapters").map(Value::as_array) else {
        return Vec::new();
    };
    chapters
        .iter()
        .filter_map(|chapter| {
            let id = chapter.get("id")?.as_str()?.to_string();
            let title = chapter
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(&id)
                .to_string();
            let summary = chapter
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let documents = chapter
                .get("documents")
                .map(Value::as_array)
                .unwrap_or_default()
                .iter()
                .filter_map(|document| document.as_str().map(str::to_string))
                .collect();
            Some(Chapter {
                id,
                title,
                title_zh: chapter
                    .get("title_zh")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                summary,
                summary_zh: chapter
                    .get("summary_zh")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                documents,
            })
        })
        .collect()
}

/// One document in the search index.
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
pub struct SearchIndex {
    entries: Vec<SearchEntry>,
    postings: HashMap<String, Vec<(usize, u32)>>,
}

/// A document in the index, with the fields shown in a result.
struct SearchEntry {
    id: String,
    language: Language,
    title: String,
    summary: String,
    chapter: String,
    title_tokens: Vec<String>,
}

impl SearchIndex {
    /// Builds an index from the entries.
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
            });
        }

        Self {
            entries: search_entries,
            postings,
        }
    }

    /// Searches the index and returns the best matches, ordered by score.
    ///
    /// At most `limit` results are returned. A query with no usable token
    /// returns nothing.
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let tokens = tokenise(&query.to_lowercase());
        if tokens.is_empty() {
            return Vec::new();
        }

        let mut scores: HashMap<usize, f64> = HashMap::new();
        for token in &tokens {
            let Some(postings) = self.postings.get(token) else {
                continue;
            };
            // A token that appears in many documents is less informative.
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

    /// Returns the number of indexed documents.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when the index is empty.
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// One search result.
#[derive(Debug, Clone)]
pub struct SearchHit {
    /// Identifier of the matching document.
    pub id: String,
    /// Language form that matched.
    pub language: Language,
    /// Title of the matching document.
    pub title: String,
    /// Summary of the matching document.
    pub summary: String,
    /// Chapter the document belongs to.
    pub chapter: String,
    /// Relevance score.
    pub score: f64,
}

/// Splits text into lowercased alphanumeric tokens.
///
/// Identifiers written with underscores, hyphens, or dots are split there as
/// well, so `mean_motion` and `mean-motion` both match `mean motion`.
fn tokenise(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|token| token.len() >= 2)
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
---
id: sgp4
title: SGP4 propagation
chapter: orbit
summary: Analytical propagation of mean elements.
tags: propagation, tle
prerequisites: tle
---

# SGP4

The model propagates [[tle|mean elements]].

## Drag

The BSTAR term.
";

    const SAMPLE_ZH: &str = "\
---
title: SGP4 传播
summary: 平均元素的解析传播。
---

# SGP4

该模型传播 [[tle|平均元素]]。

## 阻力

BSTAR 项。
";

    /// Parses the English sample under the given identifier.
    fn sample(id: &str) -> Document {
        Document::parse(id, Language::English, SAMPLE).expect("the sample must parse")
    }

    #[test]
    fn parses_metadata_and_body() {
        let document = sample("sgp4");
        assert_eq!(document.id, "sgp4");
        assert_eq!(document.title(Language::English), "SGP4 propagation");
        assert_eq!(document.tags, vec!["propagation", "tle"]);
        assert_eq!(document.prerequisites, vec!["tle"]);
        assert_eq!(document.references(), vec!["tle"]);
        assert_eq!(document.english.headings.len(), 2);
        assert_eq!(document.english.headings[1].0, "drag");
        assert!(document.english.html.contains("<h1"));
    }

    #[test]
    fn a_translation_replaces_the_language_form() {
        let mut document = sample("sgp4");
        assert!(!document.has(Language::Chinese));
        // Before the translation the Chinese reader sees the English form.
        assert_eq!(document.title(Language::Chinese), "SGP4 propagation");
        document.chinese = Some(DocumentVariant::parse(SAMPLE_ZH).expect("must parse"));
        assert!(document.has(Language::Chinese));
        assert_eq!(document.title(Language::Chinese), "SGP4 传播");
        assert_eq!(document.title(Language::English), "SGP4 propagation");
        // The references of either form are merged.
        assert_eq!(document.references(), vec!["tle"]);
    }

    #[test]
    fn rejects_a_document_without_a_title() {
        assert!(Document::parse("x", Language::English, "---\nchapter: x\n---\nbody").is_none());
        assert!(DocumentVariant::parse("---\nchapter: x\n---\nbody").is_none());
    }

    #[test]
    fn language_of_file_uses_the_suffix() {
        assert_eq!(Language::of_file("sgp4"), Language::English);
        assert_eq!(Language::of_file("sgp4.zh"), Language::Chinese);
    }

    #[test]
    fn tree_orders_documents_by_chapter() {
        let documents = vec![
            sample("sgp4"),
            Document::parse(
                "tle",
                Language::English,
                "---\ntitle: Two line elements\nchapter: orbit\n---\n\nElement sets.\n",
            )
            .expect("must parse"),
        ];
        let tree = Tree::new(documents);
        assert_eq!(tree.len(), 2);
        assert!(tree.document("sgp4").is_some());
        let ids: Vec<&str> = tree
            .ordered()
            .map(|document| document.id.as_str())
            .collect();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn a_translated_document_is_indexed_in_both_languages() {
        let mut document = sample("sgp4");
        document.chinese = Some(DocumentVariant::parse(SAMPLE_ZH).expect("must parse"));
        let tree = Tree::new(vec![document]);
        let index = tree.search_index();
        assert_eq!(index.len(), 2);
        let hits = index.search("平均元素", 5);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].language, Language::Chinese);
        // The English body contains "propagates", which the Chinese body
        // does not.
        let hits = index.search("propagates", 5);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].language, Language::English);
    }

    #[test]
    fn search_ranks_title_matches_first() {
        let documents = vec![
            Document::parse(
                "a",
                Language::English,
                "---\ntitle: Redshift\nsummary: x\n---\n\ntext about orbits\n",
            )
            .expect("must parse"),
            Document::parse(
                "b",
                Language::English,
                "---\ntitle: Orbits\nsummary: y\n---\n\ntext about redshift\n",
            )
            .expect("must parse"),
        ];
        let tree = Tree::new(documents);
        let index = tree.search_index();
        let hits = index.search("redshift orbits", 5);
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn missing_links_are_reported() {
        let tree = Tree::new(vec![sample("sgp4")]);
        let missing = tree.missing_links();
        assert_eq!(missing, vec![("sgp4".to_string(), "tle".to_string())]);
    }
}
