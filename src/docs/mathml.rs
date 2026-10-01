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
    /// A quotient drawn with a slash rather than stacked.
    Slash(Box<Node>, Box<Node>),
    /// A subscript on a subscript, as the `1` in the `v_1` of `delta_v_1`.
    Nested(Box<Node>, Box<Node>),
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
                // A name of more than one letter is an identifier too. Bare
                // text inside a row is dropped by the browser, which is why
                // a left-hand side such as `delta_v` was showing as nothing at
                // all rather than as its name.
                _ if text.chars().all(|c| c.is_alphabetic()) => {
                    format!("<mi>{}</mi>", escape(text))
                }
                // An operator is written as one rather than as bare text. A
                // browser sets text inside a row as ordinary glyphs with no
                // spacing around it, so `rev/day` came out as `revday`; an
                // operator carries its own spacing and cannot be swallowed.
                (Some(single), None) if single.is_ascii_punctuation() => {
                    format!("<mo>{}</mo>", escape(text))
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
        // A stacked fraction would be as tall as the line it is in, so a
        // quotient inside a formula is drawn with a slash and takes the
        // height of its parts.
        Node::Slash(top, bottom) => format!(
            "<mfrac linethickness=\"0\"><mrow>{}</mrow><mrow>{}</mrow></mfrac>",
            emit(top),
            emit(bottom)
        ),
        Node::Nested(base, inner) => format!(
            "<msub><mrow>{}</mrow><mrow>{}</mrow></msub>",
            emit(base),
            emit(inner)
        ),
    }
}

/// Renders a formula written in plain ASCII notation.
///
/// The documents were written before the formula support existed, and they
/// wrote formulas the way a plain text file allows: a fenced block holding
/// `delta_v_1 = sqrt( mu (2/r_1 - 1/a_t) )`. Rendering those as formulas is
/// what makes them formulas, so this reads that notation and produces the
/// same MathML as the dollar form.
///
/// The notation understood is the one the documents use:
///
/// - an identifier or a number, which may carry a subscript and a
///   superscript, as in `a_t^2` or `e^2`;
/// - `sqrt(...)` for a root, and `sin`, `cos`, `tan`, `ln`, `exp`, `floor`
///   and `arctan` among others for the functions;
/// - `*` for a product, which is written by juxtaposition;
/// - `/` for a quotient, which is drawn with a slash rather than stacked, so
///   that `2/r_1` does not become a fraction as tall as the line it is in;
/// - `+ - = < >` as they are.
///
/// The reading is a plain precedence climb: a sum, then a product, then a
/// factor. That is what makes `2/r_1 - 1/a_t` two quotients rather than one
/// quotient of the whole difference.
///
/// Returns `None` when the text is not a formula. A block holding a matrix, or
/// prose, or something not fully read is not a formula and must stay as it was
/// written: showing part of a formula that was not understood would be worse
/// than showing the source.
pub fn render_plain(text: &str) -> Option<String> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .collect();
    if lines.is_empty() {
        return None;
    }

    // A block qualifies as a formula when at least one of its lines is an
    // equation. Brackets in these documents are a matrix or a vector, which
    // this renderer does not lay out, so such a block stays a code block.
    if lines
        .iter()
        .any(|line| line.contains('[') || line.contains(']'))
        || !lines.iter().any(|line| line.contains('='))
    {
        return None;
    }

    let mut rows = String::new();
    let mut read = 0;
    for line in &lines {
        let (equation, note) = split_trailing_note(line);
        let tokens = tokenize_ascii(equation);
        let mut cursor = 0;
        let nodes = parse_sum(&tokens, &mut cursor);
        // A line that could not be read whole is still shown, but as the text
        // it was written. A block of equations often carries a continuation
        // line with no equals sign on it, and losing that would lose the
        // equation above it.
        if cursor != tokens.len() {
            rows.push_str("<mtr><mtd><mtext>");
            rows.push_str(&escape(&clean_note(equation)));
            rows.push_str("</mtext></mtd></mtr>");
            continue;
        }
        read += 1;
        rows.push_str("<mtr><mtd>");
        rows.push_str(&emit(&Node::Row(nodes)));
        if let Some(note) = note {
            // The note is a phrase about the equation rather than part of it,
            // so it is set beside the formula and not inside it.
            rows.push_str("<mspace width=\"1em\"/><mtext class=\"math-note\">");
            rows.push_str(&escape(&clean_note(note)));
            rows.push_str("</mtext>");
        }
        rows.push_str("</mtd></mtr>");
    }

    // Nothing in the block could be read, so the caller is better served by
    // the source than by a block of text set as mathematics.
    if read == 0 {
        return None;
    }

    Some(format!(
        "<span class=\"math math-display math-plain\" data-tex=\"{}\">\
         <math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"block\">\
         <mtable columnalign=\"left\">{rows}</mtable></math></span>",
        escape_attr(text.trim())
    ))
}

/// Tidies a note read out of an equation line.
///
/// The note was written in the same block as the equation, so it carries the
/// alignment of the block and runs on several lines; what is left after the
/// spaces are collapsed is a phrase.
fn clean_note(note: &str) -> String {
    note.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Splits a trailing parenthetical note from an equation.
///
/// A note is a phrase rather than part of the equation, so
/// `N = a / sqrt(1 - e^2 sin^2 phi)  (prime vertical radius)` is an equation
/// with a note and not an equation with a bracket. A parenthesis holding a
/// space is read as a note, because the expressions in these documents do not
/// write a space inside their own brackets.
fn split_trailing_note(line: &str) -> (&str, Option<&str>) {
    if !line.ends_with(')') {
        return (line, None);
    }
    match line.rfind('(') {
        Some(start) if start > 0 => {
            let inside = &line[start + 1..line.len() - 1];
            let is_phrase = inside.contains(' ')
                && !inside.contains(['=', '/', '^', '_', '(', '+'])
                && !inside.chars().any(|c| c.is_ascii_digit());
            if is_phrase {
                return (&line[..start], Some(inside));
            }
            (line, None)
        }
        _ => (line, None),
    }
}

/// One unit of a plain formula.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Ascii {
    /// An identifier: a run of letters, digits and underscores.
    Word(String),
    /// A number, possibly with a decimal point or an exponent.
    Number(String),
    /// An operator, a bracket or a mark.
    Sign(char),
}

/// Reads a plain formula into tokens.
fn tokenize_ascii(text: &str) -> Vec<Ascii> {
    let characters: Vec<char> = text.chars().collect();
    let mut tokens: Vec<Ascii> = Vec::with_capacity(characters.len());
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];
        if character.is_whitespace() {
            index += 1;
            continue;
        }
        if character.is_alphabetic() {
            let start = index;
            while index < characters.len()
                && (characters[index].is_alphanumeric()
                    || characters[index] == '_'
                    || characters[index] == '\'')
            {
                index += 1;
            }
            tokens.push(Ascii::Word(characters[start..index].iter().collect()));
            continue;
        }
        if character.is_ascii_digit() || character == '.' {
            let start = index;
            while index < characters.len() && characters[index].is_ascii_digit() {
                index += 1;
            }
            if index < characters.len() && characters[index] == '.' {
                index += 1;
                while index < characters.len() && characters[index].is_ascii_digit() {
                    index += 1;
                }
            }
            // An exponent is part of the number, as in `1e-6`.
            if index < characters.len()
                && matches!(characters[index], 'e' | 'E')
                && characters
                    .get(index + 1)
                    .is_some_and(|next| next.is_ascii_digit() || *next == '-')
            {
                index += 1;
                if characters[index] == '-' {
                    index += 1;
                }
                while index < characters.len() && characters[index].is_ascii_digit() {
                    index += 1;
                }
            }
            tokens.push(Ascii::Number(characters[start..index].iter().collect()));
            continue;
        }
        tokens.push(Ascii::Sign(character));
        index += 1;
    }
    tokens
}

/// The functions the documents write in plain ASCII.
fn ascii_function(name: &str) -> bool {
    matches!(
        name,
        "sin"
            | "cos"
            | "tan"
            | "cot"
            | "sec"
            | "csc"
            | "asin"
            | "acos"
            | "atan"
            | "arctan"
            | "sinh"
            | "cosh"
            | "tanh"
            | "ln"
            | "log"
            | "log10"
            | "exp"
            | "floor"
            | "ceil"
            | "abs"
    )
}

/// Parses a sum: factors joined by `+`, `-`, `=` or a comparison.
fn parse_sum(tokens: &[Ascii], cursor: &mut usize) -> Vec<Node> {
    let mut row = parse_product(tokens, cursor);
    while let Some(Ascii::Sign(operator @ ('+' | '-' | '=' | '<' | '>' | ','))) =
        tokens.get(*cursor)
    {
        let operator = *operator;
        *cursor += 1;
        let right = parse_product(tokens, cursor);
        row.push(Node::Text(match operator {
            '=' => " = ".to_string(),
            '<' => " < ".to_string(),
            '>' => " > ".to_string(),
            '+' => " + ".to_string(),
            // Two equations are often written on one line, separated by a
            // comma, as in `r = p / (1 + e cos nu), p = a (1 - e^2)`.
            ',' => ",  ".to_string(),
            _ => " − ".to_string(),
        }));
        row.extend(right);
    }
    row
}

/// Parses a product: factors joined by `/`, by `*`, or by nothing at all.
///
/// A product written in these documents is usually juxtaposition, as in
/// `2 f` or `mu (2/r_1 - 1/a_t)`, so a factor that follows another with
/// nothing between them continues the product. The star, where it is written,
/// is dropped.
fn parse_product(tokens: &[Ascii], cursor: &mut usize) -> Vec<Node> {
    let mut row = parse_factor(tokens, cursor);
    loop {
        match tokens.get(*cursor).cloned() {
            Some(Ascii::Sign('*')) => {
                *cursor += 1;
            }
            Some(Ascii::Sign('/')) => {
                *cursor += 1;
                let bottom = parse_factor(tokens, cursor);
                // The quotient takes the factor before it as its top and the
                // one after it as its bottom, so `2/r_1` is one quotient and
                // not a quotient of whatever happened to precede it.
                let top = row.split_off(row.len() - 1);
                row.push(Node::Slash(
                    Box::new(Node::Row(top)),
                    Box::new(Node::Row(bottom)),
                ));
            }
            // Two factors with nothing between them are a product.
            Some(Ascii::Word(_)) | Some(Ascii::Number(_)) | Some(Ascii::Sign('(')) => {
                row.extend(parse_factor(tokens, cursor));
            }
            _ => return row,
        }
    }
}

/// Parses one factor: a value with its scripts, a root, a call, or a group.
fn parse_factor(tokens: &[Ascii], cursor: &mut usize) -> Vec<Node> {
    match tokens.get(*cursor).cloned() {
        Some(Ascii::Sign(open @ ('(' | '{'))) => {
            let close = if open == '(' { ')' } else { '}' };
            *cursor += 1;
            let inner = parse_sum(tokens, cursor);
            if matches!(tokens.get(*cursor), Some(Ascii::Sign(closed)) if *closed == close) {
                *cursor += 1;
            }
            // A brace holds a group without showing brackets; a parenthesis
            // shows them, because it is written in the source.
            if open == '{' {
                vec![Node::Row(inner)]
            } else {
                vec![Node::Row(vec![
                    Node::Text("(".to_string()),
                    Node::Row(inner),
                    Node::Text(")".to_string()),
                ])]
            }
        }
        // A word in a script that is not the Latin one is a label written in
        // the line's own language, as in `近地点半径 r_p = a (1 - e)`. It is set
        // upright, because a label is not a variable.
        Some(Ascii::Word(word)) if !word.is_ascii() => {
            *cursor += 1;
            vec![Node::Text(word)]
        }
        Some(Ascii::Word(word)) => {
            *cursor += 1;
            if word == "sqrt" {
                let radicand = parse_group(tokens, cursor);
                return vec![attach_power(
                    Node::Root(Box::new(radicand), None),
                    tokens,
                    cursor,
                )];
            }
            if ascii_function(&word) {
                let inner = parse_group(tokens, cursor);
                let argument = match inner {
                    Node::Row(nodes) => Node::Row(nodes),
                    other => Node::Row(vec![other]),
                };
                let call = Node::Row(vec![
                    Node::Function(word),
                    Node::Text(" ".to_string()),
                    Node::Row(vec![
                        Node::Text("(".to_string()),
                        argument,
                        Node::Text(")".to_string()),
                    ]),
                ]);
                // A function may be raised, as in `sin^2 phi`. The power
                // belongs to the whole function rather than to its argument.
                return vec![attach_power(call, tokens, cursor)];
            }
            let base = ascii_atom(&word, tokens, cursor);
            vec![base]
        }
        Some(Ascii::Number(number)) => {
            *cursor += 1;
            let base = ascii_atom(&number, tokens, cursor);
            vec![base]
        }
        other => {
            // Any other token is read past, so that the parse always makes
            // progress rather than stalling on it.
            *cursor += 1;
            if let Some(Ascii::Sign(character)) = other {
                vec![Node::Text(character.to_string())]
            } else {
                Vec::new()
            }
        }
    }
}

/// Raises a value to the power that follows it, if there is one.
fn attach_power(value: Node, tokens: &[Ascii], cursor: &mut usize) -> Node {
    match tokens.get(*cursor) {
        Some(Ascii::Sign('^')) => {
            *cursor += 1;
            Node::Superscript(Box::new(value), Box::new(parse_group(tokens, cursor)))
        }
        _ => value,
    }
}

/// Reads a bracketed argument, or a single token when there are no brackets.
fn parse_group(tokens: &[Ascii], cursor: &mut usize) -> Node {
    match tokens.get(*cursor).cloned() {
        Some(Ascii::Sign(open @ ('(' | '{'))) => {
            let close = if open == '(' { ')' } else { '}' };
            *cursor += 1;
            let inner = parse_sum(tokens, cursor);
            if matches!(tokens.get(*cursor), Some(Ascii::Sign(closed)) if *closed == close) {
                *cursor += 1;
            }
            Node::Row(inner)
        }
        Some(Ascii::Word(word)) => {
            *cursor += 1;
            Node::Text(word)
        }
        Some(Ascii::Number(number)) => {
            *cursor += 1;
            Node::Text(number)
        }
        _ => Node::Text(String::new()),
    }
}

/// Builds the node for one identifier or number, with its scripts.
///
/// A name is written with an underscore between its parts, as in `delta_v_1`
/// or `right_ascension`, so the name is read up to the first underscore and
/// what follows is the subscript. The subscript may carry its own subscript,
/// as `v_1` does.
fn ascii_atom(word: &str, tokens: &[Ascii], cursor: &mut usize) -> Node {
    let (base, subscript) = match word.find('_') {
        Some(index) => (
            word[..index].to_string(),
            Some(word[index + 1..].to_string()),
        ),
        None => (word.to_string(), None),
    };
    let base = Node::Text(base);

    // `E_{n+1}` leaves the subscript to a brace rather than to the name, so an
    // underscore with nothing after it is completed from the next token, which
    // is the brace group that follows it.
    let subscript = match subscript {
        Some(text) if text.is_empty() => Some(match parse_group(tokens, cursor) {
            Node::Row(nodes) => Node::Row(nodes),
            other => Node::Row(vec![other]),
        }),
        Some(text) => Some(match text.rfind('_') {
            Some(index) => Node::Nested(
                Box::new(Node::Text(text[..index].to_string())),
                Box::new(Node::Text(text[index + 1..].to_string())),
            ),
            None => Node::Text(text),
        }),
        None => None,
    };

    let superscript = match tokens.get(*cursor) {
        Some(Ascii::Sign('^')) => {
            *cursor += 1;
            Some(parse_group(tokens, cursor))
        }
        _ => None,
    };

    match (subscript, superscript) {
        (None, None) => base,
        (Some(bottom), None) => Node::Subscript(Box::new(base), Box::new(bottom)),
        (None, Some(top)) => Node::Superscript(Box::new(base), Box::new(top)),
        (Some(bottom), Some(top)) => Node::Scripts(Box::new(base), Box::new(bottom), Box::new(top)),
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
}
