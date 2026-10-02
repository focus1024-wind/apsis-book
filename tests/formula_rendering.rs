//! Checks what a formula shows against what it was written as.
//!
//! A formula is written as plain text in the document and translated to
//! MathML. The translation can drop a token without dropping the formula:
//! the result still looks like a formula and is wrong. That happened here —
//! the 4800 in `year + 4800` was written as bare text inside a row, which the
//! browser does not draw, so every multi-digit number vanished from the
//! rendered page while the markup looked reasonable.
//!
//! The check is therefore on the characters rather than on the structure:
//! every letter and digit of the source must appear in the text the page
//! would show. It runs over every document rather than over a chosen sample,
//! because the defect is per formula and a sample would have missed most of
//! them.
//!
//! Only letters and digits are compared. The other characters in a formula
//! source are notation rather than content: `^` and `_` say what to raise or
//! lower, `*` says two things are multiplied, `/` says one is divided by the
//! other, and braces group. All of them are expected to disappear from the
//! page, because the layout says what they said. A number or a name has no
//! such excuse, so a missing one is a defect.
//!
//! A LaTeX command is notation of the same kind. `\sqrt` is drawn as the
//! radical sign and `\Delta` as the Greek letter, so their letters are gone
//! by design and are not missed. A command whose name is its own glyph, such
//! as `\ln`, is drawn as those letters, so stripping every command only
//! makes this check less strict about a function name and never hides a lost
//! number.

use apsis_book::docs::document::Language;

/// Returns the text a reader sees.
///
/// A browser draws the text of a leaf element and drops text that has no
/// element around it, so this counts the same and no more. An earlier
/// version counted any text inside any element, which made the check below
/// pass whatever the renderer produced: text with no element was counted as
/// drawn although the browser shows nothing.
fn visible(html: &str) -> String {
    /// The elements whose text is drawn.
    const LEAF: [&str; 6] = ["mi", "mo", "mn", "mtext", "mspace", "ms"];

    let mut out = String::new();
    // The elements currently open, innermost last.
    let mut open: Vec<String> = Vec::new();
    let mut characters = html.chars().peekable();
    let mut in_tag = false;
    let mut tag = String::new();

    for character in characters.by_ref() {
        match character {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' => {
                in_tag = false;
                let name: String = tag
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                if tag.starts_with('/') {
                    open.pop();
                } else if !tag.ends_with('/') {
                    open.push(name);
                }
            }
            _ if in_tag => tag.push(character),
            _ => {
                // Drawn only when the innermost open element holds text.
                if open
                    .last()
                    .is_some_and(|name| LEAF.contains(&name.as_str()))
                {
                    out.push(character);
                }
            }
        }
    }
    out
}

/// Returns the content characters of a formula source.
///
/// What is left out is notation: a command name, the name of an environment,
/// the `&` that separates the cells of a matrix and the `\\` that separates
/// its rows, and the braces that group. None of them is drawn as those
/// characters, so none of them is expected on the page. What remains is the
/// letters and digits, which are content and must all appear.
fn content_characters(source: &str) -> String {
    // The attribute is HTML escaped, so a cell separator arrives as `&amp;`
    // and would otherwise leave the letters of `amp` behind.
    // `sqrt` is written bare in the plain notation and with a backslash in
    // LaTeX; the bare form has to go as well, because it is drawn as a
    // radical sign rather than as its letters.
    let unescaped = source
        .replace("sqrt", "")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    let mut out = String::with_capacity(unescaped.len());
    let mut characters = unescaped.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            // A command runs to the first character that is not a letter.
            // The name is captured so that `begin` and `end` can be told
            // apart from the rest, because those two carry the name of an
            // environment in braces after them.
            '\\' => {
                let mut name = String::new();
                while let Some(next) = characters.peek().copied() {
                    if !next.is_alphabetic() {
                        break;
                    }
                    name.push(next);
                    characters.next();
                }
                if (name == "begin" || name == "end") && characters.peek() == Some(&'{') {
                    characters.next();
                    for next in characters.by_ref() {
                        if next == '}' {
                            break;
                        }
                    }
                }
            }
            '&' | '{' | '}' | '^' | '_' | '/' | '*' | '$' | '~' => {}
            other => out.push(other),
        }
    }
    out.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Returns the formula sources of one document, in order.
fn formula_blocks(html: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("data-tex=\"") {
        let after = &rest[start + "data-tex=\"".len()..];
        if let Some(end) = after.find('"') {
            found.push(after[..end].to_string());
        }
        rest = after;
    }
    found
}

#[test]
fn every_formula_shows_every_character_it_was_written_with() {
    let mut checked = 0;
    let mut failures = Vec::new();

    // The concept documents, in both languages.
    let mut sources: Vec<(String, String)> = Vec::new();
    for document in &apsis_book::docs::load_documents() {
        for language in [Language::English, Language::Chinese] {
            sources.push((document.id.clone(), document.variant(language).html.clone()));
        }
    }
    // And the interface documentation, which carries formulas of its own in
    // the doc comments of the crate it describes. Leaving it out would have
    // hidden every formula there.
    for krate in apsis_book::docs::load_api_catalog().crates {
        for item in &krate.items {
            // The doc comment is Markdown, and the service renders it with
            // the same renderer as a document.
            for text in [Some(&item.doc), item.doc_zh.as_ref()]
                .into_iter()
                .flatten()
            {
                sources.push((
                    format!("{}::{}", krate.name, item.name),
                    apsis_book::docs::markdown::render(text),
                ));
            }
        }
    }

    for (id, html) in &sources {
        let html = html.as_str();
        {
            for source in formula_blocks(html) {
                checked += 1;
                // The source as written, without the spaces that only set it
                // out on the page.
                let written = content_characters(&source);
                // The part of the page that is the formula itself: the
                // MathML and nothing else. The source is carried beside it
                // for a reader to copy, and including it here would compare
                // the source against itself and pass whatever happened.
                let from = html.find(&format!("data-tex=\"{source}\"")).unwrap_or(0);
                let start = html[from..].find("<math").map(|e| from + e).unwrap_or(from);
                let end = html[start..]
                    .find("</math>")
                    .map(|e| start + e + "</math>".len())
                    .unwrap_or(html.len());
                let shown: String = visible(&html[start..end])
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect();
                // A caret and an underscore are notation: they say what to
                // raise or lower, and neither is drawn. One that reaches the
                // output is a script that was never attached, which leaves a
                // formula that has every character in it and is still wrong.
                let body = &html[start..end];
                for marker in ["<mo>^</mo>", "<mo>_</mo>"] {
                    assert!(
                        !body.contains(marker),
                        "{id} 渲染结果里留下了记法符号 {marker}：{body}"
                    );
                }
                let missing: String = written.chars().filter(|c| !shown.contains(*c)).collect();
                if !missing.is_empty() {
                    failures.push(format!(
                        "{id}: 公式少了字符 \"{missing}\"：写入 {written}，显示 {shown}"
                    ));
                }
            }
        }
    }

    assert!(checked > 50, "只检查到 {checked} 个公式，取样过少");
    assert!(
        failures.is_empty(),
        "{} 个公式渲染后缺少字符：\n{}",
        failures.len(),
        failures.join("\n")
    );
}
