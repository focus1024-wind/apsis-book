//! Tests for the Markdown renderer.
//! Markdown 渲染器的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.
//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。

use apsis_book::docs::markdown::*;

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
    // 价格必须作为价格保留下来。
    let html = render("It costs $5 today.");
    assert!(html.contains("$5"), "the lone dollar was consumed");
    assert!(!html.contains("<math"), "no formula should be produced");
}

#[test]
fn two_prices_are_not_a_formula() {
    // The closing dollar has a space before it, which is what separates
    // this from a formula.

    // 收尾的美元符号前有一个空格，这正是它与公式的区别所在。
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
    // 网址按原样保留。
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
    // 所有片段都未知的路径按原样保留。
    let html = render_with("[`Nope`](crate::Unknown::Missing).\n", Some(&known));
    assert!(!html.contains("data-api"), "{html}");
}

/// The names of two items, for the shortcut link test.
/// 两个条目的名字，供简写链接的测试使用。
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
    // 不是条目名的标签保持为代码片段。
    let html = render_with("The range is [`0, 1`].\n", Some(&known));
    assert!(!html.contains("data-api"), "{html}");
    assert!(html.contains("<code>0, 1</code>"), "{html}");
    // A name that is not in the catalog also stays as a code span.
    // 不在目录中的名字同样保持为代码片段。
    let html = render_with("See [`NotAnItem`].\n", Some(&known));
    assert!(!html.contains("data-api"), "{html}");
}

#[test]
fn a_chinese_heading_keeps_its_own_anchor() {
    // Only ASCII letters were kept, so every Chinese heading reduced to
    // the fallback and the whole table of contents pointed at the first
    // section.

    // 只保留了 ASCII 字母，于是每个中文标题都归约到同一个回退值，整份目录都指向
    // 第一节。
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

    // rustdoc 简写保留条目名并去掉方括号，否则方括号会原样出现在一行摘要里。
    let text = to_plain_text("See [`Vec3`] and [`Epoch::mjd`](crate::Epoch::mjd).\n");
    assert!(text.contains("Vec3"));
    assert!(text.contains("mjd"));
    assert!(!text.contains('['), "{text}");
    assert!(!text.contains('`'), "{text}");
}
