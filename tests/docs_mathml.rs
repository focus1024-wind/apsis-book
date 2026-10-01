//! Tests for the formula renderer.
//!
//! They live here rather than inside the module so that `src/` holds no test
//! code, as it does in `apsis-rs`. An integration test is a separate crate and
//! reaches only what the library makes public, so the few internal items the
//! tests need are opened through `apsis_book::testkit`.

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
        "<msup><mrow><mi>x</mi></mrow><mrow>12</mrow></msup>"
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
    assert_eq!(mathml("\\notacommand"), "\\notacommand");
}

#[test]
fn a_comma_is_kept() {
    // The command is the spacing one, so dropping it would run a list of
    // symbols together. It is written as an operator so that it keeps its
    // space, which is what a comma is for.
    assert_eq!(
        mathml("\\alpha, \\beta"),
        "<mrow><mi>α</mi><mo>,</mo> <mi>β</mi></mrow>"
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
    assert!(render("x^2", false).contains("data-tex=\"x^2\""));
}

#[test]
fn the_rocket_equation_is_read() {
    // The form the documents write.
    let html = render_plain("delta_v = v_e * ln (m_0 / m_f)").expect("a formula");
    assert!(html.contains("<math"), "{html}");
    assert!(!html.contains("<msqrt>"), "no root here");
}

#[test]
fn a_subscript_within_a_subscript_is_read() {
    // `delta_v_1` is delta with a subscript v, itself carrying 1.
    let html = render_plain("delta_v_1 = 3").expect("a formula");
    assert!(html.contains("delta"), "{html}");
    assert!(html.contains("<msub>"), "the subscript was lost: {html}");
}

#[test]
fn a_power_is_read() {
    let html = render_plain("e^2 = 2 f - f^2").expect("a formula");
    assert!(html.contains("<msup>"), "the power was lost: {html}");
}

#[test]
fn a_root_is_read() {
    let html = render_plain("x = sqrt( mu / r )").expect("a formula");
    assert!(html.contains("<msqrt>"), "the root was lost: {html}");
}

#[test]
fn a_quotient_is_taken_as_its_own() {
    // `2/r_1 - 1/a_t` is two quotients, not one quotient of a difference.
    let html = render_plain("d = 2/r_1 - 1/a_t").expect("a formula");
    assert_eq!(html.matches("<mfrac").count(), 2, "{html}");
}

#[test]
fn several_equations_keep_their_order() {
    let html = render_plain("a = 1\nb = 2\nc = 3").expect("formulas");
    assert_eq!(html.matches("<mtr>").count(), 3, "{html}");
}

#[test]
fn a_matrix_stays_as_written() {
    // A matrix is not a formula this renderer lays out.
    assert!(render_plain("R = [ 1 0\n      0 1 ]").is_none());
}

#[test]
fn prose_stays_as_written() {
    assert!(render_plain("e_max such that the elevation is arctan").is_none());
    assert!(render_plain("just some text").is_none());
}

#[test]
fn a_trailing_note_is_not_part_of_the_equation() {
    // The parenthesis is a phrase, not a bracket in the expression.
    let html = render_plain("N = a / sqrt(1 - e^2 sin^2 phi)   (prime vertical radius)")
        .expect("a formula");
    // The note is set beside the formula rather than inside it, and it is
    // kept rather than dropped.
    assert!(
        html.contains("prime vertical radius"),
        "the note was lost: {html}"
    );
    assert!(
        html.contains("<mtext"),
        "the note should be set as a note: {html}"
    );
}

#[test]
fn a_bracketed_expression_is_still_read() {
    // No space inside, so it is a bracket rather than a note.
    let html = render_plain("x = sqrt( mu (2/r_1 - 1/a_t) )").expect("a formula");
    assert!(html.contains("<msqrt>"), "{html}");
}

#[test]
fn a_brace_holds_a_subscript_of_several_symbols() {
    // The Newton iteration, written the way the documents write it.
    let html = render_plain("E_{n+1} = E_n - f(E_n)").expect("a formula");
    assert!(html.contains("<msub>"), "the subscript was lost: {html}");
}

#[test]
fn a_continuation_line_is_kept_though_it_is_not_an_equation() {
    // A block of equations often carries a line with no equals sign on it.
    // Losing that line would lose the equation above it.
    let html = render_plain("JDN = day + 365 y\n      + 153 m").expect("a formula");
    assert!(html.contains("JDN"), "{html}");
    assert!(html.contains("153 m"), "the continuation was lost: {html}");
}

#[test]
fn a_block_with_no_readable_equation_is_left_alone() {
    assert!(render_plain("leap(y) = (y mod 4 == 0) and (y mod 100 != 0").is_none());
}

#[test]
fn an_absolute_value_may_hold_a_product() {
    // `| f'(x) | * | x| / | f(x) |` is one expression: a bar inside it
    // opens the next factor rather than closing the one before.
    let html = render_plain("kappa = | f'(x) | * | x| / | f(x) |").expect("a formula");
    assert!(!html.contains("<mtext"), "a note was read: {html}");
}

#[test]
fn a_power_may_be_a_word() {
    // `R^alpha` writes the exponent as a name rather than a digit.
    assert!(render_plain("gamma = k * R^alpha / (cos theta)").is_some());
}

#[test]
fn a_trailing_bracket_of_the_equation_is_not_a_note() {
    // `(cos theta)` is part of the equation even though it holds only
    // words; the whole line reads out, so there is no note.
    let html = render_plain("gamma = k * R^alpha / (cos theta)").expect("a formula");
    assert!(!html.contains("<mtext"), "a note was read: {html}");
}

#[test]
fn a_bracketed_group_is_not_mistaken_for_a_note() {
    // `(m * g)` holds an operator, so it is part of the equation; a note
    // is a phrase and has none.
    let html = render_plain("H = k * T / (m * g)").expect("a formula");
    assert!(html.contains("<mfrac"), "{html}");
    assert!(!html.contains("<mtext"), "a note was read: {html}");
}

#[test]
fn a_vector_beside_an_equation_does_not_stop_it() {
    // The block gives an equation and then the vector it acts on. The
    // equation is a formula; the vector is shown as it was written.
    let block = "a_J2 = -1.5 J2 mu R^2 / r^5 *\n       [ x (1 - 5 z^2/r^2), y (1 - 5 z^2/r^2) ]";
    let html = render_plain(block).expect("a formula");
    assert!(html.contains("<mfrac"), "the equation was lost: {html}");
    assert!(html.contains("<mtext"), "the vector was lost: {html}");
}

#[test]
fn an_operator_keeps_its_sign() {
    // The operators between two terms are written with their spacing, and
    // a browser drops bare text inside a row, which lost the sign of a
    // difference: `1 - e^2` came out as `1e^2`.
    let html = render_plain("x = (1 - e^2)^2").expect("a formula");
    // The reader writes the typographic minus, so that is what is read.
    assert!(
        html.contains("<mo>\u{2212}</mo>"),
        "the minus was lost: {html}"
    );
    assert!(html.contains("<mo>=</mo>"), "the equals was lost: {html}");
}

#[test]
fn an_equals_between_two_equations_is_kept() {
    let html = render_plain("r = p / (1 + e cos nu),  p = a (1 - e^2)").expect("a formula");
    assert!(html.contains("<mo>=</mo>"), "{html}");
    assert!(html.contains("<mo>\u{2212}</mo>"), "{html}");
}
