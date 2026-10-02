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
//! `sqrt` is notation of the same kind: it is drawn as the radical sign, so
//! its letters are gone by design and are not missed. Every other function
//! name is drawn as its own letters, as in `sin` and `ln`.

use apsis_book::docs::document::Language;

/// Returns the text a reader sees, with the markup removed.
///
/// MathML draws an element's text and drops text that has no element, so
/// keeping only what is inside an element is what the browser shows.
fn visible(html: &str) -> String {
    let mut out = String::new();
    let mut depth: usize = 0;
    let mut in_tag = false;
    let mut characters = html.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '<' => {
                in_tag = true;
                if characters.peek() == Some(&'/') {
                    depth = depth.saturating_sub(1);
                } else {
                    depth += 1;
                }
            }
            '>' => in_tag = false,
            _ if !in_tag && depth > 0 => out.push(character),
            _ => {}
        }
    }
    out
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
    let documents = apsis_book::docs::load_documents();
    let mut checked = 0;
    let mut failures = Vec::new();

    for document in &documents {
        for language in [Language::English, Language::Chinese] {
            let html = &document.variant(language).html;
            for source in formula_blocks(html) {
                checked += 1;
                // The source as written, without the spaces that only set it
                // out on the page.
                // `sqrt` is drawn as the radical sign, so its letters are
                // notation of the same kind as the operators.
                let written: String = source
                    .replace("sqrt", "")
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect();
                // The part of the page that is the formula itself.
                let start = html.find(&format!("data-tex=\"{source}\"")).unwrap_or(0);
                let end = html[start..]
                    .find("</span>")
                    .map(|e| start + e)
                    .unwrap_or(html.len());
                let shown: String = visible(&html[start..end])
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect();
                let missing: String = written.chars().filter(|c| !shown.contains(*c)).collect();
                if !missing.is_empty() {
                    failures.push(format!(
                        "{}: 公式少了字符 \"{}\"：写入 {}，显示 {}",
                        document.id, missing, written, shown
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
