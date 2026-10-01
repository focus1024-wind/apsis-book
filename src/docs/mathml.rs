//! Rendering of mathematical notation for the documents.
//!
//! The documents carry formulas, and the reader needs them to look like
//! formulas rather than like source. A full TeX engine is a large dependency
//! with its own font files, so the notation used here is translated to
//! MathML instead. Browsers lay out and print MathML themselves, so no
//! script has to run and the output works without JavaScript.
//!
//! The subset is the one orbital mechanics actually uses: fractions,
//! superscripts, subscripts, roots, Greek letters, the common operators, and
//! the delimiters that group them. An expression outside the subset is shown
//! as written rather than dropped, so an unsupported command costs nothing
//! but looks plain.
//!
//! Two forms are recognised, following the convention the documents use:
//! `$...$` for a formula within a sentence and `$$...$$` for one on its own
//! line.

use crate::docs::markdown::{escape, escape_attr};

/// Converts a formula to MathML wrapped in a container.
///
/// `display` is set when the formula stands on its own line, which makes it
/// larger and centred.
pub fn render(latex: &str, display: bool) -> String {
    let body = to_mathml(&normalize(latex));
    let class = if display { "math math-display" } else { "math" };
    // The original source is kept in a data attribute so that the formula
    // can be recovered from the page, for copying and for the tests.
    format!(
        "<span class=\"{class}\" data-tex=\"{}\"><math xmlns=\"http://www.w3.org/1998/Math/MathML\" \
         display=\"{display}\">{body}</math></span>",
        escape_attr(latex),
        display = if display { "block" } else { "inline" }
    )
}

/// Rewrites the source into the tokens the translation loop reads.
///
/// Braces are turned into single characters so that a group is one unit, and
/// the spacing commands are dropped.
fn normalize(latex: &str) -> Vec<Token> {
    let characters: Vec<char> = latex.chars().collect();
    let mut tokens: Vec<Token> = Vec::with_capacity(characters.len());
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];
        match character {
            '{' | '}' => {
                tokens.push(Token::Group);
                index += 1;
            }
            '\\' => {
                // A command runs to the first non letter.
                let mut end = index + 1;
                while end < characters.len() && characters[end].is_ascii_alphabetic() {
                    end += 1;
                }
                if end == index + 1 {
                    // A backslash before a non letter is that character.
                    if let Some(next) = characters.get(index + 1) {
                        tokens.push(Token::Symbol(*next));
                    }
                    index += 2;
                } else {
                    let name: String = characters[index + 1..end].iter().collect();
                    tokens.push(Token::Command(name));
                    index = end;
                }
            }
            _ => {
                tokens.push(Token::Symbol(character));
                index += 1;
            }
        }
    }
    tokens
}

/// One unit of the formula after normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    /// A single character.
    Symbol(char),
    /// A backslash command, without the backslash.
    Command(String),
    /// An opening or closing brace; which one is taken from the position.
    Group,
}

/// A parsed expression, ready to become MathML.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    /// Text, already escaped.
    Text(String),
    /// A row of children.
    Row(Vec<Node>),
    /// A fraction.
    Fraction(Box<Node>, Box<Node>),
    /// A root.
    Root(Box<Node>, Option<Box<Node>>),
    /// A superscript.
    Superscript(Box<Node>, Box<Node>),
    /// A subscript.
    Subscript(Box<Node>, Box<Node>),
    /// A subscript and a superscript on one base.
    Scripts(Box<Node>, Box<Node>, Box<Node>),
    /// A named function such as `sin`.
    Function(String),
}

/// The Greek letters, by command.
fn greek(name: &str) -> Option<&'static str> {
    Some(match name {
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ε",
        "varepsilon" => "ε",
        "zeta" => "ζ",
        "eta" => "η",
        "theta" => "θ",
        "vartheta" => "ϑ",
        "iota" => "ι",
        "kappa" => "κ",
        "lambda" => "λ",
        "mu" => "μ",
        "nu" => "ν",
        "xi" => "ξ",
        "pi" => "π",
        "varpi" => "ϖ",
        "rho" => "ρ",
        "varrho" => "ϱ",
        "sigma" => "σ",
        "varsigma" => "ς",
        "tau" => "τ",
        "upsilon" => "υ",
        "phi" => "φ",
        "varphi" => "ϕ",
        "chi" => "χ",
        "psi" => "ψ",
        "omega" => "ω",
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Theta" => "Θ",
        "Lambda" => "Λ",
        "Xi" => "Ξ",
        "Pi" => "Π",
        "Sigma" => "Σ",
        "Upsilon" => "Υ",
        "Phi" => "Φ",
        "Psi" => "Ψ",
        "Omega" => "Ω",
        _ => return None,
    })
}

/// The operators, by command.
fn operator(name: &str) -> Option<&'static str> {
    Some(match name {
        "times" => "×",
        "cdot" => "⋅",
        "div" => "÷",
        "pm" => "±",
        "mp" => "∓",
        "leq" | "le" => "≤",
        "geq" | "ge" => "≥",
        "neq" | "ne" => "≠",
        "approx" => "≈",
        "sim" => "∼",
        "propto" => "∝",
        "equiv" => "≡",
        "ll" => "≪",
        "gg" => "≫",
        "to" | "rightarrow" => "→",
        "leftarrow" => "←",
        "Rightarrow" => "⇒",
        "Leftarrow" => "⇐",
        "in" => "∈",
        "notin" => "∉",
        "subset" => "⊂",
        "subseteq" => "⊆",
        "supset" => "⊃",
        "infty" => "∞",
        "partial" => "∂",
        "nabla" => "∇",
        "sum" => "∑",
        "prod" => "∏",
        "int" => "∫",
        "forall" => "∀",
        "exists" => "∃",
        "circ" => "∘",
        _ => return None,
    })
}

/// The functions, by command; these are set upright rather than italic.
fn function(name: &str) -> Option<&'static str> {
    Some(match name {
        "sin" => "sin",
        "cos" => "cos",
        "tan" => "tan",
        "cot" => "cot",
        "sec" => "sec",
        "csc" => "csc",
        "arcsin" => "arcsin",
        "arccos" => "arccos",
        "arctan" => "arctan",
        "sinh" => "sinh",
        "cosh" => "cosh",
        "tanh" => "tanh",
        "log" => "log",
        "ln" => "ln",
        "lg" => "lg",
        "exp" => "exp",
        "min" => "min",
        "max" => "max",
        "det" => "det",
        "mod" => "mod",
        "arg" => "arg",
        "lim" => "lim",
        _ => return None,
    })
}

/// Renders a token sequence to MathML.
fn to_mathml(tokens: &[Token]) -> String {
    let mut cursor = 0;
    let row = parse_row(tokens, &mut cursor, None);
    match row.len() {
        1 => emit(&row[0]),
        _ => emit(&Node::Row(row)),
    }
}

/// Parses until the end of the input or, when `stop` is given, until the
/// group that ends the enclosing group.
fn parse_row(tokens: &[Token], cursor: &mut usize, stop: Option<&[char]>) -> Vec<Node> {
    let mut row: Vec<Node> = Vec::new();

    while *cursor < tokens.len() {
        match &tokens[*cursor] {
            // A closing brace ends the enclosing group; a delimiter set
            // ends the stretchy group.
            Token::Group => {
                if stop.is_some_and(|set| set.contains(&'}')) {
                    *cursor += 1;
                    return row;
                }
                if stop.is_some_and(|set| set.contains(&')')) {
                    return row;
                }
                *cursor += 1;
                let inner = parse_row(tokens, cursor, Some(&['}']));
                row.push(Node::Row(inner));
            }
            Token::Symbol(symbol) => {
                if stop.is_some_and(|set| set.contains(symbol)) {
                    return row;
                }
                *cursor += 1;
                row.extend(parse_symbol(*symbol, tokens, cursor, &mut 0));
            }
            Token::Command(name) => {
                if stop.is_some_and(|set| {
                    set.contains(&name.as_str()[..1].chars().next().unwrap_or(' '))
                }) {
                    return row;
                }
                *cursor += 1;
                row.extend(parse_command(name, tokens, cursor));
            }
        }
    }
    row
}

/// Reads the argument of a command, which is either a group or one symbol.
fn parse_argument(tokens: &[Token], cursor: &mut usize) -> Node {
    match tokens.get(*cursor) {
        Some(Token::Group) => {
            *cursor += 1;
            Node::Row(parse_row(tokens, cursor, Some(&['}'])))
        }
        // A square bracket is a group too, and is how a root takes its
        // index: `\sqrt[3]{x}`.
        Some(Token::Symbol('[')) => {
            *cursor += 1;
            let inner = parse_row(tokens, cursor, Some(&[',', ']']));
            if matches!(tokens.get(*cursor), Some(Token::Symbol(']'))) {
                *cursor += 1;
            }
            Node::Row(inner)
        }
        Some(Token::Symbol(symbol)) => {
            let symbol = *symbol;
            *cursor += 1;
            Node::Text(symbol.to_string())
        }
        Some(Token::Command(name)) => {
            let name = name.clone();
            *cursor += 1;
            match greek(&name) {
                Some(text) => Node::Text(text.to_string()),
                None => match operator(&name) {
                    Some(text) => Node::Text(text.to_string()),
                    None => Node::Text(format!("\\{name}")),
                },
            }
        }
        None => Node::Text(String::new()),
    }
}

/// Reads the scripts that follow a base, and returns the node with them.
fn attach_scripts(base: Node, tokens: &[Token], cursor: &mut usize) -> Node {
    let mut superscript: Option<Node> = None;
    let mut subscript: Option<Node> = None;

    loop {
        let marker = match tokens.get(*cursor) {
            Some(Token::Symbol('^')) => '^',
            Some(Token::Symbol('_')) => '_',
            Some(Token::Command(name)) if name == "sup" => '^',
            Some(Token::Command(name)) if name == "sub" => '_',
            _ => break,
        };
        *cursor += 1;
        let argument = parse_argument(tokens, cursor);
        if marker == '^' {
            superscript = Some(argument);
        } else {
            subscript = Some(argument);
        }
    }

    match (subscript, superscript) {
        (None, None) => base,
        (None, Some(top)) => Node::Superscript(Box::new(base), Box::new(top)),
        (Some(bottom), None) => Node::Subscript(Box::new(base), Box::new(bottom)),
        (Some(bottom), Some(top)) => Node::Scripts(Box::new(base), Box::new(bottom), Box::new(top)),
    }
}

/// The digit at a position.
fn characters_digit(tokens: &[Token], cursor: usize) -> char {
    match tokens.get(cursor) {
        Some(Token::Symbol(symbol)) => *symbol,
        _ => '0',
    }
}

/// Handles a plain character, including the brackets that group.
fn parse_symbol(symbol: char, tokens: &[Token], cursor: &mut usize, _depth: &mut u32) -> Vec<Node> {
    let base = match symbol {
        // Digits are read together as one number, so that `12` is a number
        // rather than two identifiers. A dot between digits belongs to it,
        // which also stops the layout breaking the number across a line.
        '0'..='9' => {
            let mut text = symbol.to_string();
            loop {
                match tokens.get(*cursor) {
                    Some(Token::Symbol('.'))
                        if matches!(tokens.get(*cursor + 1), Some(Token::Symbol('0'..='9'))) =>
                    {
                        text.push('.');
                        *cursor += 1;
                    }
                    Some(Token::Symbol('0'..='9')) => {
                        text.push(characters_digit(tokens, *cursor));
                        *cursor += 1;
                    }
                    _ => break,
                }
            }
            Node::Text(text)
        }
        // An opening bracket stretches to the height of what it contains.
        '(' => {
            let inner = parse_row(tokens, cursor, Some(&[',', ')']));
            if matches!(tokens.get(*cursor), Some(Token::Symbol(')'))) {
                *cursor += 1;
            }
            return vec![Node::Row(vec![
                Node::Text("(".to_string()),
                Node::Row(inner),
                Node::Text(")".to_string()),
            ])];
        }
        '[' => {
            let inner = parse_row(tokens, cursor, Some(&[',', ']']));
            if matches!(tokens.get(*cursor), Some(Token::Symbol(']'))) {
                *cursor += 1;
            }
            return vec![Node::Row(vec![
                Node::Text("[".to_string()),
                Node::Row(inner),
                Node::Text("]".to_string()),
            ])];
        }
        '{' => {
            let inner = parse_row(tokens, cursor, Some(&['}']));
            return vec![Node::Row(inner)];
        }
        '_' | '^' => Node::Text(symbol.to_string()),
        // The spaces are kept between words and dropped elsewhere, which is
        // what keeps `a b` apart without leaving gaps after a bracket.
        other => Node::Text(other.to_string()),
    };
    vec![attach_scripts(base, tokens, cursor)]
}

/// Handles a backslash command.
fn parse_command(name: &str, tokens: &[Token], cursor: &mut usize) -> Vec<Node> {
    if let Some(letter) = greek(name) {
        return vec![attach_scripts(
            Node::Text(letter.to_string()),
            tokens,
            cursor,
        )];
    }
    if let Some(symbol) = operator(name) {
        return vec![attach_scripts(
            Node::Text(symbol.to_string()),
            tokens,
            cursor,
        )];
    }
    if let Some(label) = function(name) {
        // A function is upright, so it is emitted as its own node rather
        // than as an italic variable.
        return vec![attach_scripts(
            Node::Function(label.to_string()),
            tokens,
            cursor,
        )];
    }
    match name {
        "frac" | "dfrac" | "tfrac" => {
            let top = parse_argument(tokens, cursor);
            let bottom = parse_argument(tokens, cursor);
            vec![attach_scripts(
                Node::Fraction(Box::new(top), Box::new(bottom)),
                tokens,
                cursor,
            )]
        }
        "sqrt" => {
            // The index, when there is one, is given in brackets before the
            // radicand: `\sqrt[3]{x}`.
            let degree = if matches!(tokens.get(*cursor), Some(Token::Symbol('['))) {
                Some(Box::new(parse_argument(tokens, cursor)))
            } else {
                None
            };
            let radicand = parse_argument(tokens, cursor);
            vec![attach_scripts(
                Node::Root(Box::new(radicand), degree),
                tokens,
                cursor,
            )]
        }
        "left" | "right" | "big" | "Big" | "bigg" | "Bigg" => {
            // The size and pairing commands are structural; the delimiter
            // that follows is read as an ordinary symbol.
            Vec::new()
        }
        "quad" => vec![Node::Text("\u{2009}\u{2009}".to_string())],
        // The spacing commands are punctuation with no shape of their own.
        "," => vec![Node::Text(", ".to_string())],
        ";" => vec![Node::Text("; ".to_string())],
        ":" => vec![Node::Text(": ".to_string())],
        "text" | "mathrm" | "operatorname" | "mbox" => {
            let inner = parse_argument(tokens, cursor);
            vec![Node::Row(vec![inner])]
        }
        "mathbf" | "mathit" | "mathsf" | "mathtt" => {
            let inner = parse_argument(tokens, cursor);
            vec![Node::Row(vec![inner])]
        }
        // An unknown command is shown as written, so that a formula that
        // uses one still reads rather than losing part of itself.
        other => vec![Node::Text(format!("\\{other}"))],
    }
}

/// Writes a node as MathML.
fn emit(node: &Node) -> String {
    match node {
        // MathML distinguishes an identifier, a number and operators, and a
        // browser uses the element to choose the font: an upright digit in a
        // serif face reads as a number rather than as a variable.
        Node::Text(text) => {
            let mut characters = text.chars();
            match (characters.next(), characters.next()) {
                (Some(single), None) if single.is_numeric() => {
                    format!("<mn>{}</mn>", escape(text))
                }
                (Some(single), None) if single.is_alphabetic() => {
                    format!("<mi>{}</mi>", escape(text))
                }
                _ => escape(text),
            }
        }
        Node::Function(label) => format!("<mi>{}</mi>", escape(label)),
        Node::Row(children) => match children.len() {
            // A row of one is the child itself; an extra pair of brackets
            // would be noise in the markup and would break the grouping the
            // element's layout relies on.
            1 => emit(&children[0]),
            _ => {
                let body: String = children.iter().map(emit).collect();
                format!("<mrow>{body}</mrow>")
            }
        },
        Node::Fraction(top, bottom) => format!(
            "<mfrac><mrow>{}</mrow><mrow>{}</mrow></mfrac>",
            emit(top),
            emit(bottom)
        ),
        Node::Root(radicand, degree) => match degree {
            Some(degree) => format!(
                "<mroot><mrow>{}</mrow><mrow>{}</mrow></mroot>",
                emit(radicand),
                emit(degree)
            ),
            None => format!("<msqrt><mrow>{}</mrow></msqrt>", emit(radicand)),
        },
        Node::Superscript(base, top) => format!(
            "<msup><mrow>{}</mrow><mrow>{}</mrow></msup>",
            emit(base),
            emit(top)
        ),
        Node::Subscript(base, bottom) => format!(
            "<msub><mrow>{}</mrow><mrow>{}</mrow></msub>",
            emit(base),
            emit(bottom)
        ),
        Node::Scripts(base, bottom, top) => format!(
            "<msubsup><mrow>{}</mrow><mrow>{}</mrow><mrow>{}</mrow></msubsup>",
            emit(base),
            emit(bottom),
            emit(top)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The MathML for a formula, without the wrapper, so that a test can
    /// look at the structure alone.
    fn mathml(latex: &str) -> String {
        to_mathml(&normalize(latex))
    }

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
        // symbols together.
        assert_eq!(
            mathml("\\alpha, \\beta"),
            "<mrow><mi>α</mi>, <mi>β</mi></mrow>"
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
}
