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
    /// Section headings, in order.
    pub headings: Vec<Heading>,
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
    /// Free form tags in the language of the English form.
    pub tags: Vec<String>,
    /// The same tags in Chinese, when the document declares them.
    pub tags_zh: Vec<String>,
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
    pub fn parse(id: &str, layer: &str, language: Language, text: &str) -> Option<Self> {
        let variant = DocumentVariant::parse(text)?;
        if language == Language::Chinese {
            // A Chinese file on its own is not a document; it is merged with
            // the English one by the loader.
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
    ///
    /// A leading top level heading is removed when it repeats the title. The
    /// page already shows the title, from the metadata, in the heading of the
    /// article, so a body that opens with the same heading would show it
    /// twice. A document whose first heading is a different string keeps it,
    /// because then the two headings say different things.
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
fn strip_leading_title(body: &str, title: &str) -> String {
    let mut lines = body.lines().peekable();
    // Skip leading blank lines.
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
pub fn tags_of(text: &str) -> Vec<String> {
    list_of(text, "tags")
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

/// Returns the layer a document belongs to, from the path it was loaded
/// from.
///
/// The hierarchy follows the layers of the library, so a document is placed
/// in the directory named for its layer and the loader reads the layer from
/// there. A file that sits directly in the concept directory belongs to the
/// introduction.
pub fn layer_of(relative: &str) -> String {
    match relative.split_once('/') {
        Some((layer, _)) => layer.to_string(),
        None => "overview".to_string(),
    }
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
#[derive(Debug, Clone)]
pub struct Heading {
    /// The anchor before the uniqueness suffix, used to detect a repeat.
    pub base: String,
    /// The anchor the document carries for this heading.
    pub anchor: String,
    /// The heading text.
    pub text: String,
    /// The heading level, from one to six.
    pub level: u32,
}

/// Returns a URL friendly slug, matching the anchors the Markdown renderer
/// emits.
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
#[derive(Debug, Clone)]
pub struct Group {
    /// Group identifier.
    pub id: String,
    /// Display title.
    pub title: String,
    /// Display title in Chinese, when the chapter list gives one.
    pub title_zh: Option<String>,
    /// The layers or sections of the group, in reading order.
    pub sections: Vec<Chapter>,
}

/// A chapter, or layer, of a group.
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
    groups: Vec<Group>,
    order: Vec<String>,
}

impl Tree {
    /// Builds the tree from the embedded content.
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
            let section = groups
                .iter_mut()
                .flat_map(|group| group.sections.iter_mut())
                .find(|section| section.id == layer);
            match section {
                Some(section) => section.documents.push(id),
                None => {
                    let section = Chapter {
                        id: layer.clone(),
                        title: title_case(&layer),
                        title_zh: None,
                        summary: String::new(),
                        summary_zh: None,
                        documents: vec![id],
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
        for group in &mut groups {
            for section in &mut group.sections {
                section.documents.retain(|id| map.contains_key(id));
            }
            group
                .sections
                .retain(|section| !section.documents.is_empty());
        }
        groups.retain(|group| !group.sections.is_empty());

        let order: Vec<String> = groups
            .iter()
            .flat_map(|group| group.sections.iter())
            .flat_map(|section| section.documents.iter().cloned())
            .collect();

        Self {
            documents: map,
            groups,
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

    /// Returns the groups, each holding its layers.
    pub fn groups(&self) -> &[Group] {
        &self.groups
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

/// Loads the groups and their layers from `content/concepts/chapters.json`.
///
/// The file is the single place that says what exists and in what order. The
/// directory tree says which layer a document belongs to, so the two are
/// cross-checked rather than repeated: a document listed under one layer and
/// stored in another is a mistake the reader would not see.
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
        id,
    })
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
    /// The searchable text, lowercased, kept for the substring fallback that
    /// a query of a single character uses.
    haystack: String,
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
fn tokenise(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut han: Vec<char> = Vec::new();

    // Flushes the pending Latin word.
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
fn is_han(character: char) -> bool {
    matches!(character as u32,
        0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff)
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

    /// The layer the sample documents declare in the tests.
    const LAYER: &str = "orbit";

    /// Parses the English sample under the given identifier.
    fn sample(id: &str) -> Document {
        Document::parse(id, LAYER, Language::English, SAMPLE).expect("the sample must parse")
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
        assert_eq!(document.english.headings[1].anchor, "drag");
        assert!(document.english.html.contains("<h1"));
    }

    #[test]
    fn a_repeated_top_level_title_is_dropped_from_the_body() {
        let document = sample("sgp4");
        // The sample body opens with `# SGP4`, which differs from the
        // metadata title `SGP4 propagation`, so it is kept.
        assert!(document.english.html.contains("<h1"));
        assert!(
            document
                .english
                .headings
                .iter()
                .any(|heading| heading.text == "SGP4")
        );

        // A body that repeats the metadata title loses the repeated heading.
        let repeated = "---\ntitle: SGP4 propagation\n---\n\n# SGP4 propagation\n\nBody.\n";
        let variant = DocumentVariant::parse(repeated).expect("must parse");
        assert!(!variant.html.contains("<h1"));
        assert!(
            !variant
                .headings
                .iter()
                .any(|heading| heading.text == "SGP4 propagation")
        );
        assert!(variant.html.contains("Body."));
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
        // Without a title the document cannot be listed.
        assert!(
            Document::parse("x", LAYER, Language::English, "---\nsummary: s\n---\nbody").is_none()
        );
        assert!(DocumentVariant::parse("---\nsummary: s\n---\nbody").is_none());
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
                LAYER,
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
    fn tokenise_emits_words_and_han_pairs() {
        assert_eq!(tokenise("mean_motion"), vec!["mean", "motion"]);
        assert_eq!(tokenise("轨道传播"), vec!["轨道", "道传", "传播"]);
        assert_eq!(tokenise("坐标"), vec!["坐标"]);
        // A lone Chinese character produces no token; the search falls back
        // to a substring match for it.
        assert!(tokenise("轨").is_empty());
        // A single Latin character is dropped.
        assert!(tokenise("a").is_empty());
    }

    #[test]
    fn a_chinese_query_matches_a_word_inside_a_sentence() {
        let documents = vec![
            Document::parse(
                "a",
                LAYER,
                Language::English,
                "---\ntitle: Orbits\nsummary: s\n---\n\n本条论述轨道坐标的转换。\n",
            )
            .expect("must parse"),
            Document::parse(
                "b",
                LAYER,
                Language::English,
                "---\ntitle: Time\nsummary: s\n---\n\n这里讨论时间尺度。\n",
            )
            .expect("must parse"),
        ];
        let tree = Tree::new(documents);
        let index = tree.search_index();
        // A two character query matches the pair inside the sentence.
        let hits = index.search("坐标", 5);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "a");
        // A one character query has no token and falls back to a substring
        // match.
        let hits = index.search("轨", 5);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "a");
    }

    #[test]
    fn search_ranks_title_matches_first() {
        let documents = vec![
            Document::parse(
                "a",
                LAYER,
                Language::English,
                "---\ntitle: Redshift\nsummary: x\n---\n\ntext about orbits\n",
            )
            .expect("must parse"),
            Document::parse(
                "b",
                LAYER,
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
