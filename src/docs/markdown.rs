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
//! - concept cross references written as `[[id]]` or `[[id|label]]`, which
//!   render as a link with a `data-concept` attribute so the frontend can
//!   route them without a page load
//!
//! Every text node is escaped, so a document cannot inject markup. The only
//! HTML the renderer emits is the markup it generates itself.

/// Renders a Markdown document to HTML.
pub fn render(markdown: &str) -> String {
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
            let id = slug(text);
            output.push_str(&format!(
                "<h{level} id=\"{}\">{}<a class=\"anchor\" href=\"#{id}\" aria-hidden=\"true\">#</a></h{level}>\n",
                escape(&id),
                inline(text),
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
            output.push_str(&render_table(&block));
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
            output.push_str(&render(&inner.join("\n")));
            output.push_str("</blockquote>\n");
            index = next;
            continue;
        }

        if is_unordered_item(trimmed) {
            let (block, next) =
                collect_while(&lines, index, |line| is_unordered_item(line.trim_end()));
            output.push_str(&render_list(&block, false));
            index = next;
            continue;
        }

        if is_ordered_item(trimmed) {
            let (block, next) =
                collect_while(&lines, index, |line| is_ordered_item(line.trim_end()));
            output.push_str(&render_list(&block, true));
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
        output.push_str(&format!("<p>{}</p>\n", inline(&text)));
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
fn render_table(block: &[&str]) -> String {
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
        output.push_str(&inline(cell.trim().trim_end_matches('^')));
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
            output.push_str(&inline(cell.trim().trim_end_matches('^')));
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
fn render_list(block: &[&str], ordered: bool) -> String {
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
        output.push_str(&format!("<li>{}</li>\n", inline(&content)));
    }
    output.push_str(&format!("</{tag}>\n"));
    output
}

/// Renders inline markup within a block of text.
///
/// The scan is single pass. A backtick span ends at the next backtick, a
/// link ends at its closing parenthesis, and a cross reference ends at its
/// closing bracket; anything unmatched is left as literal text.
fn inline(text: &str) -> String {
    let characters: Vec<char> = text.chars().collect();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];

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
                output.push_str(&render_concept_link(&inner));
                index = end + 2;
                continue;
            }
        }

        if character == '[' {
            if let Some((label, url, end)) = parse_link(&characters, index) {
                output.push_str(&format!(
                    "<a href=\"{}\">{}</a>",
                    escape_attr(&url),
                    inline(&label),
                ));
                index = end;
                continue;
            }
        }

        if character == '*' {
            if characters.get(index + 1) == Some(&'*') {
                if let Some(end) = find_double(&characters, index + 2) {
                    let strong: String = characters[index + 2..end].iter().collect();
                    output.push_str(&format!("<strong>{}</strong>", inline(&strong)));
                    index = end + 2;
                    continue;
                }
            } else if let Some(end) = find(&characters, index + 1, '*') {
                if end > index + 1 {
                    let emphasis: String = characters[index + 1..end].iter().collect();
                    output.push_str(&format!("<em>{}</em>", inline(&emphasis)));
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
fn render_concept_link(inner: &str) -> String {
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
        inline(label),
    )
}

/// Escapes text for use in an HTML text node.
fn escape(text: &str) -> String {
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
fn escape_attr(text: &str) -> String {
    escape(text)
}

/// Returns a URL friendly slug for a heading.
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
}
