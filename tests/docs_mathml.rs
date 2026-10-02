//! Tests for the formula renderer.
//! 公式渲染器的测试。
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

//!
//! 它们放在这里而不是模块内部，使 `src/` 不含测试代码，与 `apsis-rs` 一致。
//! 集成测试是独立的 crate，只能触达库公开的部分，因此测试所需的那少数内部条目
//! 经 `apsis_book::testkit` 开放。

use apsis_book::docs::mathml::*;
use apsis_book::testkit::mathml;

#[test]
fn a_variable_is_an_identifier() {
    assert_eq!(mathml("x"), "<mi>x</mi>");
}

#[test]
fn a_power_becomes_a_superscript() {
    assert_eq!(
        mathml("x^2"),
        "<msup><mrow><mi>x</mi></mrow><mrow><mn>2</mn></mrow></msup>"
    );
}

#[test]
fn a_group_holds_several_tokens() {
    assert_eq!(
        mathml("x^{12}"),
        "<msup><mrow><mi>x</mi></mrow><mrow><mn>12</mn></mrow></msup>"
    );
}

#[test]
fn a_fraction_has_a_numerator_and_a_denominator() {
    assert_eq!(
        mathml("\\frac{a}{b}"),
        "<mfrac><mrow><mi>a</mi></mrow><mrow><mi>b</mi></mrow></mfrac>"
    );
}

#[test]
fn a_greek_letter_is_translated() {
    assert_eq!(mathml("\\theta"), "<mi>θ</mi>");
}

#[test]
fn an_unknown_command_is_kept_as_written() {
    // Losing part of a formula would be worse than showing it plainly.
    // Text is given an element even when it is not understood, because text
    // without one is not drawn at all.

    // 丢掉公式的一部分比把它朴素地显示出来更糟。即使用户未被理解，文本仍会被赋予
    // 元素，因为没有元素的文本根本不会被绘制。
    assert_eq!(mathml("\\notacommand"), "<mtext>\\notacommand</mtext>");
}

#[test]
fn a_comma_is_kept() {
    // The command is the spacing one, so dropping it would run a list of
    // symbols together. It is written as an operator so that it keeps its
    // space, which is what a comma is for.

    // 该命令是空格类命令，丢掉它会让一串符号挤在一起。它被写成运算符，从而保住
    // 自己的空格——逗号正是为此而设。
    assert_eq!(
        mathml("\\alpha, \\beta"),
        "<mrow><mi>α</mi><mo>,</mo><mspace width=\"0.25em\"/><mi>β</mi></mrow>"
    );
}

#[test]
fn a_root_can_have_an_index() {
    let html = mathml("\\sqrt[3]{x}");
    assert!(html.contains("<mroot>"), "the index was lost: {html}");
}

#[test]
fn text_is_escaped() {
    assert!(mathml("a<b").contains("&lt;"));
}

#[test]
fn a_display_formula_is_marked_as_such() {
    let html = render("x^2", true);
    assert!(html.contains("math-display"));
    assert!(html.contains("display=\"block\""));
}

#[test]
fn the_source_is_kept_in_the_output() {
    // The page keeps the source so the formula can be copied back.
    // 页面保留源码，使公式能被复制回去。
    assert!(render("x^2", false).contains("data-tex=\"x^2\""));
}

#[test]
fn a_bracketed_group_takes_the_power_that_follows_it() {
    // The bracket branches returned before the scripts were attached, so the
    // caret was set as an ordinary symbol and the exponent sat beside the
    // group: `(a + b)^2` showed its own caret.

    // 括号分支在挂接上下标之前就返回了，于是脱字符被当作普通符号排版，指数则
    // 落在括号组旁边：`(a + b)^2` 把自己的脱字符显示了出来。
    for latex in [
        r"(a + b)^2",
        r"\left( \frac{x_{ke}}{n_0} \right)^{2/3}",
        r"\left[ a \right]^2",
        r"\{a\}_2",
    ] {
        let html = render_plain_of(latex);
        assert!(
            html.contains("<msup>") || html.contains("<msub>"),
            "{latex} kept its script marker: {html}"
        );
        assert!(
            !html.contains("<mo>^</mo>") && !html.contains("<mo>_</mo>"),
            "{latex} left a script marker in the output: {html}"
        );
    }
}

/// The MathML of a formula written in LaTeX.
/// 以 LaTeX 书写的公式的 MathML。
fn render_plain_of(latex: &str) -> String {
    apsis_book::docs::mathml::render(latex, false)
        .split("display=\"inline\">")
        .nth(1)
        .unwrap_or("")
        .split("</math>")
        .next()
        .unwrap_or("")
        .to_string()
}
