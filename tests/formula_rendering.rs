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
//!
//! 比对公式显示的与它被书写的内容。
//!
//! 公式在文档中写成纯文本，再翻译为 MathML。翻译可能丢掉一个 token 而公式仍
//! 在：结果看起来仍像公式，却是错的。这里就发生过——`year + 4800` 里的 4800
//! 被写成行内裸文本，而浏览器不绘制它，于是渲染出的页面上所有多位数都不见了，
//! 标记看起来却还算合理。
//!
//! 因此本检查针对字符而不是结构：源码中的每一个字母与数字都必须出现在页面会
//! 显示的文本里。它遍历每一篇文档而不是抽取样本，因为该缺陷是逐个公式的，样本
//! 会漏掉大部分。
//!
//! 只比对字母与数字。公式源码中的其他字符是记法而非内容：`^` 与 `_` 说明升降，
//! `*` 表示相乘，`/` 表示相除，花括号用于分组。它们都应当从页面上消失，因为
//! 排版已经表达了它们的意思。而数字或名字没有这样的借口，少一个就是缺陷。
//!
//! LaTeX 命令是同一类记法。`\sqrt` 被画成根号，`\Delta` 被画成希腊字母，
//! 因此它们的字母本就消失，不算缺失。名字即其字形的命令，例如 `\ln`，就按那些
//! 字母绘制，所以剥掉所有命令只会让本检查对函数名更宽松，而绝不会掩盖一个丢失
//! 的数字。

use apsis_book::docs::document::Language;

/// Returns the text a reader sees.
///
/// A browser draws the text of a leaf element and drops text that has no
/// element around it, so this counts the same and no more. An earlier
/// version counted any text inside any element, which made the check below
/// pass whatever the renderer produced: text with no element was counted as
/// drawn although the browser shows nothing.
///
/// 返回读者看到的文本。
///
/// 浏览器绘制叶子元素内的文本，并丢弃没有元素包裹的文本，因此这里只按同样的
/// 规则统计。早先的版本统计任何元素内部的文本，使下面的检查无论在渲染器产出
/// 什么时都通过：没有元素的文本被算作已绘制，而浏览器什么也不显示。
fn visible(html: &str) -> String {
    /// The elements whose text is drawn.
    /// 其文本会被绘制的元素。
    const LEAF: [&str; 6] = ["mi", "mo", "mn", "mtext", "mspace", "ms"];

    let mut out = String::new();
    // The elements currently open, innermost last.
    // 当前打开的元素，最内层在最后。
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
                // 只有当最内层打开的元素持有文本时才绘制。
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
///
/// 返回公式源码中的内容字符。
///
/// 被剔除的是记法：命令名、环境名、分隔矩阵单元格的 `&` 与分隔各行的 `\\`，
/// 以及用于分组的花括号。它们都不以这些字符的形式被绘制，因此都不应出现在
/// 页面上。剩下的是字母与数字，它们是内容，必须全部出现。
fn content_characters(source: &str) -> String {
    // The attribute is HTML escaped, so a cell separator arrives as `&amp;`
    // and would otherwise leave the letters of `amp` behind.
    // `sqrt` is written bare in the plain notation and with a backslash in
    // LaTeX; the bare form has to go as well, because it is drawn as a
    // radical sign rather than as its letters.

    // 该属性经过 HTML 转义，因此单元格分隔符到手时是 `&amp;`，否则会留下
    // `amp` 这几个字母。`sqrt` 在朴素记法中写成裸词、在 LaTeX 中带反斜杠；
    // 裸词形式同样要去掉，因为它被绘制成根号而不是那些字母。
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

            // 命令一直延伸到第一个非字母字符。名字被捕获下来，是为了把
            // `begin` 与 `end` 同其余命令区分开——只有这两个会在其后用花括号
            // 携带环境名。
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
/// 按顺序返回一篇文档中的公式源码。
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
    // 概念文档，含两种语言。
    let mut sources: Vec<(String, String)> = Vec::new();
    for document in &apsis_book::docs::load_documents() {
        for language in [Language::English, Language::Chinese] {
            sources.push((document.id.clone(), document.variant(language).html.clone()));
        }
    }
    // And the interface documentation, which carries formulas of its own in
    // the doc comments of the crate it describes. Leaving it out would have
    // hidden every formula there.

    // 以及接口文档，它在其所描述的 crate 的文档注释中自带公式。把它排除在外会
    // 让那里的每一个公式都无人检查。
    for module in apsis_book::docs::load_api_catalog().modules {
        for item in &module.items {
            // The doc comment is Markdown, and the service renders it with
            // the same renderer as a document.

            // 文档注释是 Markdown，服务用与文档相同的渲染器渲染它。
            for text in [Some(&item.doc), item.doc_zh.as_ref()]
                .into_iter()
                .flatten()
            {
                sources.push((
                    format!("{}::{}", module.name, item.name),
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

                // 源码按书写时的样子，去掉那些仅为页面排版而设的空格。
                let written = content_characters(&source);
                // The part of the page that is the formula itself: the
                // MathML and nothing else. The source is carried beside it
                // for a reader to copy, and including it here would compare
                // the source against itself and pass whatever happened.

                // 页面上属于公式本身的部分：MathML，别无其他。源码被携带在其旁
                // 供读者复制，若把它算进来，就成了源码与自身比对，无论发生什么
                // 都会通过。
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

                // 脱字符与下划线是记法：它们说明升降，二者都不被绘制。出现在
                // 输出里的那一个，说明有上下标从未被挂接，这会留下一个每个字符
                // 都在、却依然是错的公式。
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
