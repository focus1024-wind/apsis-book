//! A small Markdown renderer for the concept documents.
//!
//! The renderer covers the subset of Markdown the content uses. It is not a
//! general implementation of the CommonMark specification; it is the
//! smallest renderer that expresses the documents, and it is written here
//! because the service may not depend on a crate.
//!
//! Supported constructs:
//!
//! - ATX headings, `#` through `######`
//! - paragraphs, with soft line breaks joined by a space
//! - fenced code blocks delimited by three backticks, with an optional
//!   language name
//! - pipe tables with a delimiter row, where a cell may end with `^` to
//!   mark it as right aligned
//! - unordered lists marked with `-` and ordered lists marked with `1.`
//! - block quotes marked with `>`
//! - thematic breaks written as `---`
//! - inline emphasis `*text*` and strong emphasis `**text**`
//! - inline code spans delimited by backticks
//! - links written as `[text](url)`
//!
//! A link whose target is a Rust path, such as `[Epoch](crate::Epoch)`,
//! names an item of the library rather than a web address. Such a link is
//! rendered as a link to the interface page of the item, so that a doc
//! comment can cross reference another item the way `rustdoc` does.
//! - concept cross references written as `[[id]]` or `[[id|label]]`, which
//!   render as a link with a `data-concept` attribute so the frontend can
//!   route them without a page load
//!
//! Every text node is escaped, so a document cannot inject markup. The only
//! HTML the renderer emits is the markup it generates itself.

/// Renders a Markdown document to HTML.
pub fn render(markdown: &str) -> String {
    render_with(markdown, None)
}

/// Renders a Markdown document, resolving a link to a library item.
///
/// `items` is the set of item names the interface catalog contains. A
/// shortcut link whose name is not in the set is left as a code span, so a
/// doc comment can write `[`0, 1`]` without producing a link to nothing.
pub fn render_with(markdown: &str, items: Option<&std::collections::HashSet<String>>) -> String {
    // Anchors must be unique within the document: two headings that reduce
    // to the same text would otherwise share one anchor, and the second
    // entry of a table of contents would scroll to the first.
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let lines: Vec<&str> = markdown.lines().collect();
    let mut output = String::with_capacity(markdown.len() + markdown.len() / 2);
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_end();

        if trimmed.trim().is_empty() {
            index += 1;
            continue;
        }

        if let Some(language) = fence_language(trimmed) {
            index += 1;
            let mut code = String::new();
            while index < lines.len() && !lines[index].trim_start().starts_with("```") {
                code.push_str(lines[index]);
                code.push('\n');
                index += 1;
            }
            index += 1; // closing fence
            output.push_str(&render_code_block(language, &code));
            continue;
        }

        if let Some(level) = heading_level(trimmed) {
            let text = trimmed[level..].trim();
            // Two headings that reduce to the same text must not share one
            // anchor, or every later entry of a table of contents scrolls
            // to the first of them.
            let base = slug(text);
            let count = seen.entry(base.clone()).or_insert(0);
            *count += 1;
            let id = if *count == 1 {
                base
            } else {
                format!("{base}-{count}")
            };
            output.push_str(&format!(
                "<h{level} id=\"{}\">{}<a class=\"anchor\" href=\"#{id}\" aria-hidden=\"true\">#</a></h{level}>\n",
                escape(&id),
                inline_items(text, items),
            ));
            index += 1;
            continue;
        }

        if is_thematic_break(trimmed) {
            output.push_str("<hr>\n");
            index += 1;
            continue;
        }

        if trimmed.starts_with('|') {
            let (block, next) = collect_table(&lines, index);
            output.push_str(&render_table(&block, items));
            index = next;
            continue;
        }

        if trimmed.starts_with("> ") || trimmed == ">" {
            let (block, next) =
                collect_while(&lines, index, |line| line.trim_start().starts_with('>'));
            let inner: Vec<&str> = block
                .iter()
                .map(|line| line.trim_start().trim_start_matches('>').trim_start())
                .collect();
            output.push_str("<blockquote>\n");
            output.push_str(&render_with(&inner.join("\n"), items));
            output.push_str("</blockquote>\n");
            index = next;
            continue;
        }

        if is_unordered_item(trimmed) {
            let (block, next) =
                collect_while(&lines, index, |line| is_unordered_item(line.trim_end()));
            output.push_str(&render_list(&block, false, items));
            index = next;
            continue;
        }

        if is_ordered_item(trimmed) {
            let (block, next) =
                collect_while(&lines, index, |line| is_ordered_item(line.trim_end()));
            output.push_str(&render_list(&block, true, items));
            index = next;
            continue;
        }

        // A paragraph runs until a blank line or the start of another block.
        let (block, next) = collect_while(&lines, index, |line| {
            let trimmed = line.trim_end();
            !trimmed.trim().is_empty()
                && fence_language(trimmed).is_none()
                && heading_level(trimmed).is_none()
                && !trimmed.starts_with('|')
                && !trimmed.starts_with("> ")
                && !is_thematic_break(trimmed)
                && !is_unordered_item(trimmed)
                && !is_ordered_item(trimmed)
        });
        let text = block
            .iter()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" ");
        output.push_str(&format!("<p>{}</p>\n", inline_items(&text, items)));
        index = next;
    }

    output
}

/// Returns the language of a fence line, or `None` when the line is not a
/// fence.
fn fence_language(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("```")?;
    Some(rest.trim())
}

/// Returns the heading level of a line, or `None` when it is not a heading.
fn heading_level(line: &str) -> Option<usize> {
    let mut level = 0;
    for character in line.chars() {
        if character == '#' {
            level += 1;
        } else {
            break;
        }
    }
    if (1..=6).contains(&level) && line[level..].starts_with(' ') {
        Some(level)
    } else {
        None
    }
}

/// Returns `true` when a line is a thematic break.
fn is_thematic_break(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 3 && trimmed.chars().all(|character| character == '-')
}

/// Returns `true` when a line opens an unordered list item.
fn is_unordered_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- ") || trimmed == "-"
}

/// Returns `true` when a line opens an ordered list item.
fn is_ordered_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    let digits: String = trimmed.chars().take_while(char::is_ascii_digit).collect();
    !digits.is_empty() && trimmed[digits.len()..].starts_with(". ")
}

/// Collects lines while `predicate` holds.
fn collect_while<'a, F>(lines: &[&'a str], start: usize, predicate: F) -> (Vec<&'a str>, usize)
where
    F: Fn(&str) -> bool,
{
    let mut block = Vec::new();
    let mut index = start;
    while index < lines.len() && predicate(lines[index]) {
        block.push(lines[index]);
        index += 1;
    }
    (block, index)
}

/// Collects the lines of a table, which includes the delimiter row.
fn collect_table<'a>(lines: &[&'a str], start: usize) -> (Vec<&'a str>, usize) {
    collect_while(lines, start, |line| {
        let trimmed = line.trim_end();
        trimmed.starts_with('|') && !trimmed.trim().is_empty()
    })
}

/// Renders a fenced code block.
fn render_code_block(language: &str, code: &str) -> String {
    let class = if language.is_empty() {
        String::new()
    } else {
        format!(" class=\"language-{}\"", escape(language))
    };
    format!(
        "<figure class=\"code\"><figcaption>{}</figcaption><pre><code{}>{}</code></pre></figure>\n",
        if language.is_empty() {
            "text"
        } else {
            language
        },
        class,
        escape(code),
    )
}

/// Renders a pipe table.
///
/// The first row is the header, the second row is the delimiter that marks
/// the table, and the remaining rows are the body. A cell whose content ends
/// with `^` is aligned to the right, which is used for numeric columns.
fn render_table(block: &[&str], items: Option<&std::collections::HashSet<String>>) -> String {
    if block.len() < 2 {
        return String::new();
    }
    let header = split_row(block[0]);
    let aligns: Vec<bool> = split_row(block[1])
        .iter()
        .map(|cell| cell.trim().ends_with('^'))
        .collect();

    let mut output = String::from("<div class=\"table-wrap\"><table>\n<thead>\n<tr>");
    for (index, cell) in header.iter().enumerate() {
        let align = aligns.get(index).copied().unwrap_or(false);
        output.push_str("<th");
        if align {
            output.push_str(" class=\"numeric\"");
        }
        output.push('>');
        output.push_str(&inline_items(cell.trim().trim_end_matches('^'), items));
        output.push_str("</th>");
    }
    output.push_str("</tr>\n</thead>\n<tbody>\n");
    for row in &block[2..] {
        output.push_str("<tr>");
        for (index, cell) in split_row(row).iter().enumerate() {
            let align = aligns.get(index).copied().unwrap_or(false);
            output.push_str("<td");
            if align {
                output.push_str(" class=\"numeric\"");
            }
            output.push('>');
            output.push_str(&inline_items(cell.trim().trim_end_matches('^'), items));
            output.push_str("</td>");
        }
        output.push_str("</tr>\n");
    }
    output.push_str("</tbody>\n</table></div>\n");
    output
}

/// Splits a table row into its cells.
fn split_row(row: &str) -> Vec<String> {
    let trimmed = row.trim().trim_start_matches('|').trim_end_matches('|');
    trimmed.split('|').map(str::to_string).collect()
}

/// Renders a list.
fn render_list(
    block: &[&str],
    ordered: bool,
    items: Option<&std::collections::HashSet<String>>,
) -> String {
    let tag = if ordered { "ol" } else { "ul" };
    let mut output = format!("<{tag}>\n");
    for line in block {
        let trimmed = line.trim_start();
        let content = if ordered {
            let digits: String = trimmed.chars().take_while(char::is_ascii_digit).collect();
            trimmed[digits.len() + 2..].to_string()
        } else {
            trimmed[2..].to_string()
        };
        output.push_str(&format!("<li>{}</li>\n", inline_items(&content, items)));
    }
    output.push_str(&format!("</{tag}>\n"));
    output
}

/// Renders inline markup within a block of text.
///
/// The scan is single pass. A backtick span ends at the next backtick, a
/// link ends at its closing parenthesis, and a cross reference ends at its
/// closing bracket; anything unmatched is left as literal text.
#[allow(dead_code)]
fn inline(text: &str) -> String {
    inline_items(text, None)
}

/// Renders a formula written between dollar signs.
///
/// The formula is found before any other inline markup, because a dollar
/// sign is otherwise ordinary text and a formula may itself contain the
/// characters that the other rules act on.
/// Reports whether a position falls inside a code span.
///
/// The span is found by counting the backticks before this position: an odd
/// number means the last one opened a span that has not been closed. Code
/// spans do not nest, so the count alone settles it.
fn inside_code_span(characters: &[char], index: usize) -> bool {
    let mut backticks = 0;
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        if characters[cursor] == '`' {
            backticks += 1;
        }
        // A span ends at a line break, so a backtick on an earlier line
        // cannot pair with one on this one.
        if characters[cursor] == '\n' {
            break;
        }
    }
    backticks % 2 == 1
}

/// Finds a formula at `start`, if there is one.
///
/// Returns the source, whether it stands on its own line, and the position
/// just past it.
///
/// An inline formula may not begin or end with a space. That rule is what
/// separates a formula from a price or a shell prompt: in `$5 and `$HOME`
/// the second dollar has a space before it, so the pair is not a formula and
/// both dollars are text. Without the rule a sentence mentioning two prices
/// would be read as a formula between them.
fn find_formula(characters: &[char], start: usize) -> Option<(String, bool, usize)> {
    // Two dollars open a formula that stands on its own line; a single one
    // opens a formula within the sentence.
    let display = characters.get(start + 1) == Some(&'$');
    let open = if display { start + 2 } else { start + 1 };

    if display {
        // A display formula runs to the closing pair and may span lines.
        let mut cursor = open;
        while cursor + 1 < characters.len() {
            // A code span is stepped over, for the same reason as below.
            if characters[cursor] == '`' {
                match find(characters, cursor + 1, '`') {
                    Some(end) => {
                        cursor = end + 1;
                        continue;
                    }
                    None => {
                        cursor += 1;
                        continue;
                    }
                }
            }
            if characters[cursor] == '$' && characters[cursor + 1] == '$' {
                let source: String = characters[open..cursor].iter().collect();
                if source.trim().is_empty() {
                    return None;
                }
                return Some((source, true, cursor + 2));
            }
            cursor += 1;
        }
        return None;
    }

    // An inline formula must not open on a space, and must not close on one.
    match characters.get(open) {
        Some(character) if character.is_whitespace() => return None,
        None => return None,
        _ => {}
    }
    let mut cursor = open + 1;
    while cursor < characters.len() {
        // A code span is stepped over rather than read, so that a dollar
        // inside one cannot close the formula. This is what keeps
        // `` `$HOME` `` from being read as the end of one.
        if characters[cursor] == '`' {
            match find(characters, cursor + 1, '`') {
                Some(end) => {
                    cursor = end + 1;
                    continue;
                }
                // An unmatched backtick is ordinary text.
                None => {
                    cursor += 1;
                    continue;
                }
            }
        }
        if characters[cursor] == '$' {
            if characters[cursor - 1].is_whitespace() {
                return None;
            }
            let source: String = characters[open..cursor].iter().collect();
            return Some((source, false, cursor + 1));
        }
        // A newline ends the sentence, and a formula does not cross one.
        if characters[cursor] == '\n' {
            return None;
        }
        cursor += 1;
    }
    None
}

/// Renders inline markup, resolving a shortcut link against `items`.
fn inline_items(text: &str, items: Option<&std::collections::HashSet<String>>) -> String {
    let characters: Vec<char> = text.chars().collect();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];

        // A formula is read before anything else, because a dollar sign is
        // otherwise ordinary text and the characters inside a formula are
        // not escaped. A dollar inside a code span is left to the code rule,
        // which is the older one and the more specific of the two.
        if character == '$' && !inside_code_span(&characters, index) {
            match find_formula(&characters, index) {
                Some((source, display, next)) => {
                    output.push_str(&crate::docs::mathml::render(&source, display));
                    index = next;
                    continue;
                }
                // An unpaired dollar is a dollar.
                None => {
                    output.push('$');
                    index += 1;
                    continue;
                }
            }
        }

        if character == '`' {
            if let Some(end) = find(&characters, index + 1, '`') {
                let code: String = characters[index + 1..end].iter().collect();
                output.push_str("<code>");
                output.push_str(&escape(&code));
                output.push_str("</code>");
                index = end + 1;
                continue;
            }
        }

        if character == '[' && characters.get(index + 1) == Some(&'[') {
            if let Some(end) = find_pair(&characters, index + 2) {
                let inner: String = characters[index + 2..end].iter().collect();
                output.push_str(&render_concept_link(&inner, items));
                index = end + 2;
                continue;
            }
        }

        // A shortcut link, `[`Name`]`, is how a doc comment refers to
        // another item without repeating the path. It is rendered as a link
        // to the item page when the inner text names a plausible item.
        if character == '[' && characters.get(index + 1) == Some(&'`') {
            if let Some((label, end)) = parse_shortcut_link(&characters, index) {
                if let Some(item) = shortcut_item(&label, items) {
                    output.push_str(&format!(
                        "<a class=\"api-link\" data-api=\"{}\" href=\"#/api/{}\"><code>{}</code></a>",
                        escape_attr(item),
                        escape_attr(item),
                        escape(&label),
                    ));
                    index = end;
                    continue;
                }
            }
        }

        if character == '[' {
            if let Some((label, url, end)) = parse_link(&characters, index) {
                match rust_path_target(&url, items) {
                    // A link to a library item becomes a link to the item
                    // page, which is where the reader can see the signature
                    // and the rest of the documentation.
                    Some(item) => output.push_str(&format!(
                        "<a class=\"api-link\" data-api=\"{}\" href=\"#/api/{}\">{}</a>",
                        escape_attr(&item),
                        escape_attr(&item),
                        inline_items(&label, items),
                    )),
                    None => output.push_str(&format!(
                        "<a href=\"{}\">{}</a>",
                        escape_attr(&url),
                        inline_items(&label, items),
                    )),
                }
                index = end;
                continue;
            }
        }

        if character == '*' {
            if characters.get(index + 1) == Some(&'*') {
                if let Some(end) = find_double(&characters, index + 2) {
                    let strong: String = characters[index + 2..end].iter().collect();
                    output.push_str(&format!(
                        "<strong>{}</strong>",
                        inline_items(&strong, items)
                    ));
                    index = end + 2;
                    continue;
                }
            } else if let Some(end) = find(&characters, index + 1, '*') {
                if end > index + 1 {
                    let emphasis: String = characters[index + 1..end].iter().collect();
                    output.push_str(&format!("<em>{}</em>", inline_items(&emphasis, items)));
                    index = end + 1;
                    continue;
                }
            }
        }

        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            other => output.push(other),
        }
        index += 1;
    }

    output
}

/// Finds the next occurrence of `needle` at or after `start`.
fn find(characters: &[char], start: usize, needle: char) -> Option<usize> {
    (start..characters.len()).find(|index| characters[*index] == needle)
}

/// Finds the next `]]` at or after `start`.
fn find_pair(characters: &[char], start: usize) -> Option<usize> {
    (start..characters.len().saturating_sub(1))
        .find(|index| characters[*index] == ']' && characters[*index + 1] == ']')
}

/// Finds the next `**` at or after `start`.
fn find_double(characters: &[char], start: usize) -> Option<usize> {
    (start..characters.len().saturating_sub(1))
        .find(|index| characters[*index] == '*' && characters[*index + 1] == '*')
}

/// Parses a shortcut link, `[`text`]`, starting at `start`.
///
/// Returns the text between the backticks and the index just past the
/// closing bracket.
fn parse_shortcut_link(characters: &[char], start: usize) -> Option<(String, usize)> {
    if characters.get(start) != Some(&'[') || characters.get(start + 1) != Some(&'`') {
        return None;
    }
    let tick = find(characters, start + 2, '`')?;
    if characters.get(tick + 1) != Some(&']') {
        return None;
    }
    let label: String = characters[start + 2..tick].iter().collect();
    Some((label, tick + 2))
}

/// Returns the item name a shortcut link refers to.
///
/// A shortcut link names an item, optionally with a module path or a
/// method, such as `Epoch::mjd` or `apsis_time::Epoch`. The item page
/// routes on the item name, so the last segment is used. A label that is
/// not a plausible item name, such as `0, 1` or `-pi/2, pi/2`, is not a
/// link and returns `None`.
fn shortcut_item<'a>(
    label: &'a str,
    items: Option<&std::collections::HashSet<String>>,
) -> Option<&'a str> {
    let name = label.rsplit("::").next()?.trim();
    let is_name = !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_');
    if !is_name {
        return None;
    }
    // When the catalog is available the name must be an item of the
    // library, which is what keeps a label such as `0, 1` or a phrase from
    // becoming a link. Without it, a name that starts with an upper case
    // letter is taken as an item, which is the correct guess for a doc
    // comment that names another item.
    match items {
        Some(items) => items.contains(name).then_some(name),
        None => name
            .chars()
            .next()
            .is_some_and(char::is_uppercase)
            .then_some(name),
    }
}

/// Returns the item name of a Rust path written as a link target.
///
/// A doc comment links to a library item with a Rust path, such as
/// `crate::Epoch`, `core::result::Result`, or `f64::rem_euclid`. The name
/// the interface page routes on is the last segment of the path, with any
/// generic arguments removed. A target that is a web address or a fragment
/// is not a Rust path and returns `None`, which leaves the link as written.
///
/// A path may end in a variant or a method rather than the item itself,
/// such as `crate::PropagationErrorKind::Decayed`. The resolver walks the
/// path from the right and returns the first segment that is a known item,
/// so such a link points at the enum or the trait that declares it.
fn rust_path_target(
    url: &str,
    items: Option<&std::collections::HashSet<String>>,
) -> Option<String> {
    if url.is_empty() || url.contains('/') || url.starts_with('#') {
        return None;
    }
    let segments: Vec<&str> = url
        .split("::")
        .map(|segment| segment.split(['<', '(']).next().unwrap_or("").trim())
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return None;
    }
    let is_rust_path =
        segments.len() > 1 || segments[0].chars().next().is_some_and(char::is_uppercase);
    if !is_rust_path {
        return None;
    }
    let is_name = |segment: &str| {
        !segment.is_empty()
            && segment
                .chars()
                .all(|character| character.is_alphanumeric() || character == '_')
    };
    match items {
        Some(items) => segments
            .iter()
            .rev()
            .find(|segment| is_name(segment) && items.contains(**segment))
            .map(|segment| (*segment).to_string()),
        None => segments
            .iter()
            .rev()
            .find(|segment| is_name(segment))
            .map(|segment| (*segment).to_string()),
    }
}

/// Parses a `[label](url)` link starting at `start`, returning the label,
/// the URL, and the index just past the link.
fn parse_link(characters: &[char], start: usize) -> Option<(String, String, usize)> {
    if characters.get(start) != Some(&'[') {
        return None;
    }
    let label_end = find(characters, start + 1, ']')?;
    if characters.get(label_end + 1) != Some(&'(') {
        return None;
    }
    let url_end = find(characters, label_end + 2, ')')?;
    let label: String = characters[start + 1..label_end].iter().collect();
    let url: String = characters[label_end + 2..url_end].iter().collect();
    Some((label, url, url_end + 1))
}

/// Renders a concept cross reference.
///
/// The argument is either `id` or `id|label`. An identifier that is not a
/// plausible identifier is rendered as literal text, so a stray `[[` in the
/// prose cannot produce a broken link.
fn render_concept_link(inner: &str, items: Option<&std::collections::HashSet<String>>) -> String {
    let (id, label) = match inner.split_once('|') {
        Some((id, label)) => (id.trim(), label.trim()),
        None => (inner.trim(), inner.trim()),
    };
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return format!("[[{}]]", escape(inner));
    }
    format!(
        "<a class=\"concept-link\" data-concept=\"{}\" href=\"#/concept/{}\">{}</a>",
        escape_attr(id),
        escape_attr(id),
        inline_items(label, items),
    )
}

/// Escapes text for use in an HTML text node.
pub(crate) fn escape(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            other => output.push(other),
        }
    }
    output
}

/// Escapes text for use inside a double quoted attribute.
///
/// Only the characters that can end the attribute or start an entity are
/// escaped; the value never contains a quote because identifiers are
/// restricted by [`render_concept_link`] and links are escaped here too.
pub(crate) fn escape_attr(text: &str) -> String {
    escape(text)
}

/// Returns a URL friendly slug for a heading.
fn slug(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut last_dash = true;
    for character in text.chars() {
        // Letters of any script are kept, so a Chinese heading keeps its own
        // characters instead of collapsing to the fallback. Keeping only the
        // ASCII ones gave every Chinese heading the same anchor, so the
        // table of contents pointed every entry at the first section.
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

/// Strips Markdown markup from text, leaving readable plain text.
///
/// This is used to build the search index, where markup would only add
/// noise. Cross references keep their label, links keep their text, and
/// code spans keep their content.
pub fn to_plain_text(markdown: &str) -> String {
    let mut output = String::with_capacity(markdown.len());
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.is_empty() {
            continue;
        }
        let mut text = trimmed.to_string();
        if let Some(level) = heading_level(&text) {
            text = text[level..].trim().to_string();
        }
        if text.starts_with('|') {
            text = split_row(&text)
                .iter()
                .map(|cell| cell.trim().trim_end_matches('^').to_string())
                .filter(|cell| !cell.is_empty() && !cell.chars().all(|c| c == '-'))
                .collect::<Vec<_>>()
                .join(" ");
        }
        let text = text.trim_start_matches(['>', '-', ' ']).trim().to_string();
        output.push_str(&strip_inline(&text));
        output.push(' ');
    }
    output
}

/// Removes inline markup from a single line.
fn strip_inline(text: &str) -> String {
    let characters: Vec<char> = text.chars().collect();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        if character == '`' {
            if let Some(end) = find(&characters, index + 1, '`') {
                output.extend(&characters[index + 1..end]);
                index = end + 1;
                continue;
            }
        }
        if character == '[' && characters.get(index + 1) == Some(&'[') {
            if let Some(end) = find_pair(&characters, index + 2) {
                let inner: String = characters[index + 2..end].iter().collect();
                let label = inner
                    .split_once('|')
                    .map(|(_, label)| label)
                    .unwrap_or(&inner);
                output.push_str(label);
                index = end + 2;
                continue;
            }
        }
        if character == '[' && characters.get(index + 1) == Some(&'`') {
            // A rustdoc shortcut reference, `[`Epoch`]`, is a reference to
            // another item. The plain text keeps the item name so that a
            // summary still names what it is about, and the brackets never
            // reach the reader.
            if let Some((label, end)) = parse_shortcut_link(&characters, index) {
                let name = label.rsplit("::").next().unwrap_or(&label);
                output.push_str(name.trim());
                index = end;
                continue;
            }
        }
        if character == '[' {
            if let Some((label, _, end)) = parse_link(&characters, index) {
                output.push_str(&label);
                index = end;
                continue;
            }
        }
        if character == '*' {
            index += 1;
            continue;
        }
        output.push(character);
        index += 1;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_inline_formula_is_rendered() {
        let html = render("The rate is $r$ per second.");
        assert!(html.contains("<math"), "the formula was not rendered");
        assert!(!html.contains("math-display"), "it belongs in the sentence");
        assert!(html.contains("per second."), "the sentence was lost");
    }

    #[test]
    fn a_display_formula_is_rendered_on_its_own() {
        let html = render("$$a^2 + b^2 = c^2$$");
        assert!(html.contains("math-display"), "the formula is not a block");
        assert!(html.contains("display=\"block\""), "it is not a display");
    }

    #[test]
    fn a_dollar_without_a_pair_is_left_alone() {
        // A price must survive as a price.
        let html = render("It costs $5 today.");
        assert!(html.contains("$5"), "the lone dollar was consumed");
        assert!(!html.contains("<math"), "no formula should be produced");
    }

    #[test]
    fn two_prices_are_not_a_formula() {
        // The closing dollar has a space before it, which is what separates
        // this from a formula.
        let html = render("It costs $5 and then $6.");
        assert!(
            !html.contains("<math"),
            "two prices should not be a formula"
        );
        assert!(html.contains("$5 and then $6."), "the prices were consumed");
    }

    #[test]
    fn a_dollar_inside_code_is_left_alone() {
        let html = render("`$HOME` is the home directory.");
        assert!(!html.contains("<math"), "code should not become a formula");
        assert!(
            html.contains("<code>$HOME</code>"),
            "the code span was lost"
        );
    }

    #[test]
    fn a_formula_does_not_cross_a_line() {
        let html = render("A $x$ and\nB $y$.");
        assert_eq!(html.matches("<math").count(), 2, "each should be its own");
    }

    #[test]
    fn a_backslash_in_a_formula_is_kept() {
        let html = render("$\\frac{1}{2}$");
        assert!(html.contains("<mfrac>"), "the fraction was not built");
    }

    #[test]
    fn renders_headings_and_paragraphs() {
        let html = render("# Title\n\nSome *text* and **more**.\n");
        assert!(html.contains("<h1 id=\"title\">Title"));
        assert!(html.contains("<p>Some <em>text</em> and <strong>more</strong>.</p>"));
    }

    #[test]
    fn renders_tables_with_alignment() {
        let markdown = "| a | b |\n|---|---|\n| 1 | 2 |\n";
        let html = render(markdown);
        assert!(html.contains("<th>a</th>"));
        assert!(html.contains("<td>1</td>"));
    }

    #[test]
    fn escapes_code_and_text() {
        let html = render("Use `a < b` and <script>.\n");
        assert!(html.contains("<code>a &lt; b</code>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn a_rust_path_link_points_at_the_interface_page() {
        let html = render("Returns [`Epoch`](crate::Epoch) and [more](https://example.com).\n");
        assert!(html.contains("data-api=\"Epoch\""), "{html}");
        assert!(html.contains("href=\"#/api/Epoch\""), "{html}");
        // A web address is left as it is.
        assert!(html.contains("href=\"https://example.com\""), "{html}");
    }

    #[test]
    fn a_path_to_a_variant_resolves_to_the_declaring_item() {
        let known: std::collections::HashSet<String> = ["PropagationErrorKind"]
            .iter()
            .map(|n| n.to_string())
            .collect();
        let html = render_with(
            "Returns [`Decayed`](crate::PropagationErrorKind::Decayed).\n",
            Some(&known),
        );
        assert!(html.contains("data-api=\"PropagationErrorKind\""), "{html}");
        // A path whose segments are all unknown is left as written.
        let html = render_with("[`Nope`](crate::Unknown::Missing).\n", Some(&known));
        assert!(!html.contains("data-api"), "{html}");
    }

    /// The names of two items, for the shortcut link test.
    fn items() -> std::collections::HashSet<String> {
        ["mjd", "Epoch"]
            .iter()
            .map(|name| name.to_string())
            .collect()
    }

    #[test]
    fn a_shortcut_link_points_at_the_interface_page() {
        let known = items();
        let html = render_with(
            "See [`Epoch::mjd`] and [`apsis_time::Epoch`].\n",
            Some(&known),
        );
        assert!(html.contains("data-api=\"mjd\""), "{html}");
        assert!(html.contains("href=\"#/api/mjd\""), "{html}");
        assert!(html.contains("data-api=\"Epoch\""), "{html}");
        // A label that is not an item name stays as a code span.
        let html = render_with("The range is [`0, 1`].\n", Some(&known));
        assert!(!html.contains("data-api"), "{html}");
        assert!(html.contains("<code>0, 1</code>"), "{html}");
        // A name that is not in the catalog also stays as a code span.
        let html = render_with("See [`NotAnItem`].\n", Some(&known));
        assert!(!html.contains("data-api"), "{html}");
    }

    #[test]
    fn a_chinese_heading_keeps_its_own_anchor() {
        // Only ASCII letters were kept, so every Chinese heading reduced to
        // the fallback and the whole table of contents pointed at the first
        // section.
        let html = render("# 时间尺度\n\n正文。\n\n# 单位约定\n\n更多。\n");
        assert!(html.contains("id=\"时间尺度\""), "{html}");
        assert!(html.contains("id=\"单位约定\""), "{html}");
        assert!(!html.contains("id=\"section\""), "{html}");
    }

    #[test]
    fn a_repeated_heading_gets_a_distinct_anchor() {
        let html = render("# 说明\n\n甲。\n\n# 说明\n\n乙。\n");
        assert!(html.contains("id=\"说明\""), "{html}");
        assert!(html.contains("id=\"说明-2\""), "{html}");
    }

    #[test]
    fn renders_concept_links() {
        let html = render("See [[sgp4]] and [[tle|the element set]].\n");
        assert!(html.contains("data-concept=\"sgp4\""));
        assert!(html.contains(">the element set</a>"));
    }

    #[test]
    fn renders_code_blocks_with_language() {
        let html = render("```rust\nlet x = 1;\n```\n");
        assert!(html.contains("class=\"language-rust\""));
        assert!(html.contains("let x = 1;"));
    }

    #[test]
    fn renders_lists_and_quotes() {
        let html = render("- one\n- two\n\n> quoted\n");
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>one</li>"));
        assert!(html.contains("<blockquote>"));
    }

    #[test]
    fn plain_text_drops_the_markup() {
        let text = to_plain_text("# Title\n\nSee [[sgp4|SGP4]] and `code`.\n");
        assert!(text.contains("Title"));
        assert!(text.contains("SGP4"));
        assert!(!text.contains("[["));
        assert!(!text.contains('`'));
    }

    #[test]
    fn plain_text_resolves_a_shortcut_reference() {
        // A rustdoc shortcut keeps the item name and loses the brackets,
        // which otherwise appear verbatim in a one line summary.
        let text = to_plain_text("See [`Vec3`] and [`Epoch::mjd`](crate::Epoch::mjd).\n");
        assert!(text.contains("Vec3"));
        assert!(text.contains("mjd"));
        assert!(!text.contains('['), "{text}");
        assert!(!text.contains('`'), "{text}");
    }
}
