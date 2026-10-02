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
//!
//! 面向概念文档的小型 Markdown 渲染器。
//!
//! 该渲染器覆盖内容所用的 Markdown 子集。它不是 CommonMark 规范的通用实现；
//! 它是能表达这些文档的最小渲染器，之所以写在这里，是因为服务不得依赖任何
//! crate。
//!
//! 支持的构造：
//!
//! - ATX 标题，`#` 至 `######`
//! - 段落，软换行以空格连接
//! - 以三个反引号界定的围栏代码块，可带语言名
//! - 带分隔行的竖线表格，单元格可以 `^` 结尾表示右对齐
//! - 以 `-` 标记的无序列表与以 `1.` 标记的有序列表
//! - 以 `>` 标记的块引用
//! - 写作 `---` 的主题分隔线
//! - 行内强调 `*text*` 与加粗强调 `**text**`
//! - 以反引号界定的行内代码片段
//! - 写作 `[text](url)` 的链接
//! - 概念交叉引用，写作 `[[id]]` 或 `[[id|label]]`，渲染为带 `data-concept`
//!   属性的链接，使前端无须整页加载即可路由
//!
//! 目标为 Rust 路径的链接，例如 `[Epoch](crate::Epoch)`，指的是库中的一个条目
//! 而不是网址。这类链接渲染为指向该条目接口页的链接，使文档注释能像 `rustdoc`
//! 那样交叉引用另一个条目。
//!
//! 每个文本节点都被转义，因此文档无法注入标记。渲染器输出的 HTML 只有它自己
//! 生成的标记。

/// Renders a Markdown document to HTML.
///
/// 把一篇 Markdown 文档渲染为 HTML。
pub fn render(markdown: &str) -> String {
    render_with(markdown, None)
}

/// Renders a Markdown document, resolving a link to a library item.
///
/// `items` is the set of item names the interface catalog contains. A
/// shortcut link whose name is not in the set is left as a code span, so a
/// doc comment can write `[`0, 1`]` without producing a link to nothing.
///
/// 渲染一篇 Markdown 文档，并解析指向库条目的链接。
///
/// `items` 是接口目录所含条目名的集合。名字不在集合中的简写链接保持为代码片段，
/// 因此文档注释可以写 `[`0, 1`]` 而不会产生一个指向空处的链接。
pub fn render_with(markdown: &str, items: Option<&std::collections::HashSet<String>>) -> String {
    // Anchors must be unique within the document: two headings that reduce
    // to the same text would otherwise share one anchor, and the second
    // entry of a table of contents would scroll to the first.

    // 锚点必须在文档内唯一：否则归约为同一文本的两个标题会共用一个锚点，目录的
    // 第二个条目会滚到第一个上。
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

            // 归约为同一文本的两个标题不得共用一个锚点，否则目录中后来的每个
            // 条目都会滚到它们中的第一个。
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

        // 段落一直延伸到空行或另一个块的开始。
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
///
/// 返回围栏行的语言；该行不是围栏时返回 `None`。
fn fence_language(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("```")?;
    Some(rest.trim())
}

/// Returns the heading level of a line, or `None` when it is not a heading.
///
/// 返回某一行的标题层级；它不是标题时返回 `None`。
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
///
/// 当某一行是主题分隔线时返回 `true`。
fn is_thematic_break(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 3 && trimmed.chars().all(|character| character == '-')
}

/// Returns `true` when a line opens an unordered list item.
///
/// 当某一行开启一个无序列表项时返回 `true`。
fn is_unordered_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- ") || trimmed == "-"
}

/// Returns `true` when a line opens an ordered list item.
///
/// 当某一行开启一个有序列表项时返回 `true`。
fn is_ordered_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    let digits: String = trimmed.chars().take_while(char::is_ascii_digit).collect();
    !digits.is_empty() && trimmed[digits.len()..].starts_with(". ")
}

/// Collects lines while `predicate` holds.
///
/// 在 `predicate` 成立期间收集各行。
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
///
/// 收集一张表格的各行，其中包含分隔行。
fn collect_table<'a>(lines: &[&'a str], start: usize) -> (Vec<&'a str>, usize) {
    collect_while(lines, start, |line| {
        let trimmed = line.trim_end();
        trimmed.starts_with('|') && !trimmed.trim().is_empty()
    })
}

/// Renders a fenced code block.
///
/// 渲染一个围栏代码块。
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
///
/// 渲染一张竖线表格。
///
/// 第一行是表头，第二行是标示该表格的分隔行，其余各行是表体。内容以 `^` 结尾的
/// 单元格右对齐，用于数值列。
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
///
/// 把一行表格拆分为它的各个单元格。
fn split_row(row: &str) -> Vec<String> {
    let trimmed = row.trim().trim_start_matches('|').trim_end_matches('|');
    trimmed.split('|').map(str::to_string).collect()
}

/// Renders a list.
///
/// 渲染一个列表。
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
///
/// 渲染一段文本内的行内标记。
///
/// 扫描是一趟式的。反引号片段在下一个反引号处结束，链接在收尾圆括号处结束，
/// 交叉引用在收尾方括号处结束；任何未配对的内容都按字面文本保留。
#[allow(dead_code)]
fn inline(text: &str) -> String {
    inline_items(text, None)
}

/// Renders a formula written between dollar signs.
///
/// The formula is found before any other inline markup, because a dollar
/// sign is otherwise ordinary text and a formula may itself contain the
/// characters that the other rules act on.
///
/// 渲染写在美元符号之间的公式。
///
/// 公式在任何其他行内标记之前被找出，因为美元符号本身只是普通文本，而公式内部
/// 可能含有其他规则会作用的字符。
/// Reports whether a position falls inside a code span.
///
/// The span is found by counting the backticks before this position: an odd
/// number means the last one opened a span that has not been closed. Code
/// spans do not nest, so the count alone settles it.
///
/// 报告某个位置是否落在代码片段之内。
///
/// 该片段通过数此位置之前的反引号来判定：奇数个意味着最后一个开启了一个尚未闭合
/// 的片段。代码片段不嵌套，因此这个计数就足以定论。
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

        // 片段在换行处结束，因此前一行上的反引号不能与本行上的配对。
        if characters[cursor] == '\n' {
            break;
        }
    }
    backticks % 2 == 1
}

/// Finds a formula at `start`, if there is one.
///
/// 在 `start` 处查找一个公式，若有。
///
/// Returns the source, whether it stands on its own line, and the position
/// just past it.
///
/// An inline formula may not begin or end with a space. That rule is what
/// separates a formula from a price or a shell prompt: in `$5 and `$HOME`
/// the second dollar has a space before it, so the pair is not a formula and
/// both dollars are text. Without the rule a sentence mentioning two prices
/// would be read as a formula between them.
///
/// 返回源码、它是否独占一行，以及紧接其后位置。
///
/// 行内公式不得以空格开始或结束。正是这条规则把公式与价格或 shell 提示符区分
/// 开来：在 `$5 和 `$HOME` 中第二个美元符号前有空格，因此这一对不是公式，两个
/// 美元符号都是文本。没有这条规则，一句提到两个价格的话会被读成它们之间的公式。
fn find_formula(characters: &[char], start: usize) -> Option<(String, bool, usize)> {
    // Two dollars open a formula that stands on its own line; a single one
    // opens a formula within the sentence.

    // 两个美元符号开启独占一行的公式；一个则开启句内的公式。
    let display = characters.get(start + 1) == Some(&'$');
    let open = if display { start + 2 } else { start + 1 };

    if display {
        // A display formula runs to the closing pair and may span lines.

        // 独占一行的公式一直延伸到收尾的一对，且可以跨行。
        let mut cursor = open;
        while cursor + 1 < characters.len() {
            // A code span is stepped over, for the same reason as below.

            // 代码片段被跳过，理由与下文相同。
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

    // 行内公式不得以空格开启，也不得以空格结束。
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

        // 代码片段被跳过而不是读取，使其中的美元符号无法闭合公式。正是这一点
        // 使 `` `$HOME` `` 不会被读作某个公式的结尾。
        if characters[cursor] == '`' {
            match find(characters, cursor + 1, '`') {
                Some(end) => {
                    cursor = end + 1;
                    continue;
                }
                // An unmatched backtick is ordinary text.

                // 未配对的反引号是普通文本。
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

        // 换行结束句子，而公式不跨行。
        if characters[cursor] == '\n' {
            return None;
        }
        cursor += 1;
    }
    None
}

/// Renders inline markup, resolving a shortcut link against `items`.
///
/// 渲染行内标记，并针对 `items` 解析简写链接。
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

        // 公式在其他一切之前被读取，因为美元符号本身只是普通文本，而公式内部的
        // 字符不被转义。代码片段内的美元符号交给代码规则处理，那是更早、也更
        // 具体的规则。
        if character == '$' && !inside_code_span(&characters, index) {
            match find_formula(&characters, index) {
                Some((source, display, next)) => {
                    output.push_str(&crate::docs::mathml::render(&source, display));
                    index = next;
                    continue;
                }
                // An unpaired dollar is a dollar.

                // 未配对的美元符号就是美元符号。
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

        // 简写链接 `[`Name`]` 是文档注释引用另一个条目而不重复路径的方式。当内部
        // 文本指名一个像样的条目时，它渲染为指向该条目页面的链接。
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

                    // 指向库条目的链接变成指向该条目页面的链接，读者在那里能看到
                    // 签名与其余的文档。
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
///
/// 在 `start` 处或其后查找 `needle` 的下一次出现。
fn find(characters: &[char], start: usize, needle: char) -> Option<usize> {
    (start..characters.len()).find(|index| characters[*index] == needle)
}

/// Finds the next `]]` at or after `start`.
///
/// 在 `start` 处或其后查找下一个 `]]`。
fn find_pair(characters: &[char], start: usize) -> Option<usize> {
    (start..characters.len().saturating_sub(1))
        .find(|index| characters[*index] == ']' && characters[*index + 1] == ']')
}

/// Finds the next `**` at or after `start`.
///
/// 在 `start` 处或其后查找下一个 `**`。
fn find_double(characters: &[char], start: usize) -> Option<usize> {
    (start..characters.len().saturating_sub(1))
        .find(|index| characters[*index] == '*' && characters[*index + 1] == '*')
}

/// Parses a shortcut link, `[`text`]`, starting at `start`.
///
/// Returns the text between the backticks and the index just past the
/// closing bracket.
///
/// 从 `start` 开始解析一个简写链接 `[`text`]`。
///
/// 返回反引号之间的文本，以及紧接收尾方括号之后的位置。
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
///
/// 返回简写链接所指的条目名。
///
/// 简写链接指名一个条目，可带模块路径或方法，例如 `Epoch::mjd` 或
/// `apsis_time::Epoch`。条目页面以条目名路由，因此取最后一段。不像样的条目名
/// ——例如 `0, 1` 或 `-pi/2, pi/2`——不是链接，返回 `None`。
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

    // 目录可用时，名字必须是库中的一个条目，这正是使 `0, 1` 这样的标签或一个
    // 短语不会变成链接的原因。没有目录时，以大写字母开头的名字被当作条目，对
    // 指名另一个条目的文档注释而言，这是正确的猜测。
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
///
/// 返回写作链接目标的 Rust 路径的条目名。
///
/// 文档注释用 Rust 路径链接到库条目，例如 `crate::Epoch`、
/// `core::result::Result` 或 `f64::rem_euclid`。接口页面路由所用的名字是该路径
/// 的最后一段，并去掉任何泛型实参。目标若是网址或片段，就不是 Rust 路径，返回
/// `None`，链接按原样保留。
///
/// 路径可以以变体或方法结尾而不是条目本身，例如
/// `crate::PropagationErrorKind::Decayed`。解析器从右往左走这条路径，返回第一个
/// 已知条目的片段，因此这样的链接指向声明它的枚举或 trait。
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
///
/// 从 `start` 开始解析 `[label](url)` 链接，返回标签、URL 与紧接链接之后的
/// 位置。
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
///
/// 渲染一个概念交叉引用。
///
/// 实参或为 `id`，或为 `id|label`。不像样的标识按字面文本渲染，因此正文中一个
/// 走失的 `[[` 不会产生断链。
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
///
/// 转义文本，供 HTML 文本节点使用。
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
///
/// 转义文本，供双引号属性内部使用。
///
/// 只转义那些能结束属性或开启实体的字符；该值绝不包含引号，因为标识受
/// [`render_concept_link`] 限制，链接在此也被转义。
pub(crate) fn escape_attr(text: &str) -> String {
    escape(text)
}

/// Returns a URL friendly slug for a heading.
///
/// 返回标题的 URL 友好短名。
fn slug(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let mut last_dash = true;
    for character in text.chars() {
        // Letters of any script are kept, so a Chinese heading keeps its own
        // characters instead of collapsing to the fallback. Keeping only the
        // ASCII ones gave every Chinese heading the same anchor, so the
        // table of contents pointed every entry at the first section.

        // 保留任何文字系统的字母，因此中文标题保留自己的字符，而不会塌缩到回退值。
        // 只保留 ASCII 字母曾使每个中文标题得到同一个锚点，目录的每一条都指向
        // 第一节。
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
///
/// 从文本中剥去 Markdown 标记，留下可读的纯文本。
///
/// 用于构建搜索索引，那里标记只会带来噪声。交叉引用保留其标签，链接保留其文本，
/// 代码片段保留其内容。
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
///
/// 从单独一行中移除行内标记。
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

            // rustdoc 简写引用 `[`Epoch`]` 是对另一个条目的引用。纯文本保留条目
            // 名，使摘要仍然指名它讲的是什么，而方括号绝不会到达读者眼前。
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
