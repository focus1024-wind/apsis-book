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
    // The delimiters of a display formula are on their own lines, so the
    // source arrives with the newline that followed the opening pair.
    let latex = latex.trim();
    let body = to_mathml(&normalize(latex));
    let class = if display { "math math-display" } else { "math" };
    // The original source is kept in a data attribute so that the formula
    // can be recovered from the page, for copying and for the tests.
    // The MathML is one element per token, and a browser copying a selection
    // walks them one at a time: a formula copied from the page arrives as one
    // character per line. The source is therefore carried beside it in an
    // element a reader can select, and the MathML is taken out of selection
    // so that what a copy yields is the formula as it was written.
    format!(
        "<span class=\"{class}\" data-tex=\"{source}\"><math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"{display}\">{body}</math><span class=\"math-source\">{source}</span></span>",
        source = escape_attr(latex),
        display = if display { "block" } else { "inline" }
    )
}

/// Rewrites the source into the tokens the translation loop reads.
///
/// Braces are turned into single characters so that a group is one unit, and
/// the spacing commands are dropped.
pub(crate) fn normalize(latex: &str) -> Vec<Token> {
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
                    // A backslash before a mark is a command of its own when
                    // the mark is a spacing one, and that character itself
                    // otherwise.
                    match characters.get(index + 1) {
                        Some(',') => tokens.push(Token::Command("thinspace".to_string())),
                        Some(';') => tokens.push(Token::Command("thickspace".to_string())),
                        Some(':') => tokens.push(Token::Command("medspace".to_string())),
                        Some('!') => tokens.push(Token::Command("negspace".to_string())),
                        Some(' ') => tokens.push(Token::Command("space".to_string())),
                        Some(next) => tokens.push(Token::Symbol(*next)),
                        None => {}
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
pub(crate) enum Token {
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
    /// A mark written above its argument, as in `\dot\Omega`.
    Overscript(Box<Node>, String),
    /// Words that stand as they are written, as in `\text{km}`.
    Verbatim(String),
    /// A matrix, with the delimiters its environment asks for.
    Matrix {
        /// The opening and closing delimiters, which may be empty.
        open: &'static str,
        close: &'static str,
        /// The cells, one row at a time.
        rows: Vec<Vec<Node>>,
    },
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
pub(crate) fn operator(name: &str) -> Option<&'static str> {
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
pub(crate) fn to_mathml(tokens: &[Token]) -> String {
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

/// Returns the delimiters a matrix environment is drawn with.
fn matrix_delimiters(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "matrix" | "aligned" | "align" | "gathered" | "array" => ("", ""),
        "pmatrix" => ("(", ")"),
        "bmatrix" => ("[", "]"),
        "vmatrix" => ("|", "|"),
        "Vmatrix" => ("‖", "‖"),
        "cases" => ("{", ""),
        _ => return None,
    })
}

/// Reads the name of an environment from the group that follows `\begin`.
fn parse_environment_name(tokens: &[Token], cursor: &mut usize) -> String {
    let mut name = String::new();
    if !matches!(tokens.get(*cursor), Some(Token::Group)) {
        return name;
    }
    *cursor += 1;
    while let Some(token) = tokens.get(*cursor) {
        if token == &Token::Group {
            break;
        }
        if let Token::Symbol(character) = token {
            name.push(*character);
        }
        *cursor += 1;
    }
    if matches!(tokens.get(*cursor), Some(Token::Group)) {
        *cursor += 1;
    }
    name
}

/// Reads the rows of a matrix, up to the `\end` that closes it.
///
/// A cell ends at `&`, a row ends at `\\`, and the environment ends at
/// `\end`. The tokens of a cell are read by the same handlers as anywhere
/// else, so a fraction or a root inside a cell works as it does outside.
fn parse_matrix_rows(tokens: &[Token], cursor: &mut usize) -> Vec<Vec<Node>> {
    // Two accumulators: the cell being read, and the cells of the row so
    // far. One alone does not do, because `&` ends a cell and starts the
    // next, and the nodes after it belong to the next cell rather than
    // beside the one just closed.
    let mut rows: Vec<Vec<Node>> = Vec::new();
    let mut cells: Vec<Node> = Vec::new();
    let mut cell: Vec<Node> = Vec::new();

    let end_cell = |cells: &mut Vec<Node>, cell: &mut Vec<Node>| {
        cells.push(Node::Row(std::mem::take(cell)));
    };

    loop {
        match tokens.get(*cursor) {
            None => break,
            Some(Token::Command(name)) if name == "end" => break,
            Some(Token::Symbol('\\')) => {
                *cursor += 1;
                end_cell(&mut cells, &mut cell);
                rows.push(std::mem::take(&mut cells));
            }
            Some(Token::Symbol('&')) => {
                *cursor += 1;
                end_cell(&mut cells, &mut cell);
            }
            Some(Token::Group) => {
                *cursor += 1;
                let inner = parse_row(tokens, cursor, Some(&['}']));
                cell.push(Node::Row(inner));
            }
            Some(Token::Symbol(symbol)) => {
                let symbol = *symbol;
                *cursor += 1;
                cell.extend(parse_symbol(symbol, tokens, cursor, &mut 0));
            }
            Some(Token::Command(name)) => {
                let name = name.clone();
                *cursor += 1;
                cell.extend(parse_command(&name, tokens, cursor));
            }
        }
    }
    // The last row has no `\\` after it.
    if !cell.is_empty() || !cells.is_empty() || rows.is_empty() {
        end_cell(&mut cells, &mut cell);
        rows.push(cells);
    }
    rows
}

/// Reads the characters of an argument as the text they are written with.
///
/// Used where the argument is words rather than mathematics, so that the
/// letters are not read as variables one at a time.
fn parse_raw_text(tokens: &[Token], cursor: &mut usize) -> String {
    let mut out = String::new();
    if !matches!(tokens.get(*cursor), Some(Token::Group)) {
        return out;
    }
    *cursor += 1;
    while let Some(token) = tokens.get(*cursor) {
        match token {
            Token::Group => break,
            Token::Symbol(character) => out.push(*character),
            Token::Command(name) => out.push_str(name),
        }
        *cursor += 1;
    }
    if matches!(tokens.get(*cursor), Some(Token::Group)) {
        *cursor += 1;
    }
    out
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
        //
        // A bracketed group is a base like any other, so it is given its
        // scripts by the same call at the end of this function. Returning
        // the group from here instead left `(a + b)^2` with the caret set as
        // an ordinary symbol and the exponent beside it, and inside a
        // fraction the exponent escaped the fraction altogether.
        '(' => {
            let inner = parse_row(tokens, cursor, Some(&[',', ')']));
            if matches!(tokens.get(*cursor), Some(Token::Symbol(')'))) {
                *cursor += 1;
            }
            Node::Row(vec![
                Node::Text("(".to_string()),
                Node::Row(inner),
                Node::Text(")".to_string()),
            ])
        }
        '[' => {
            let inner = parse_row(tokens, cursor, Some(&[',', ']']));
            if matches!(tokens.get(*cursor), Some(Token::Symbol(']'))) {
                *cursor += 1;
            }
            Node::Row(vec![
                Node::Text("[".to_string()),
                Node::Row(inner),
                Node::Text("]".to_string()),
            ])
        }
        '{' => Node::Row(parse_row(tokens, cursor, Some(&['}']))),
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
        // A matrix environment: \begin{name} cells \end{name}, with `&`
        // between the cells of a row and `\\` between the rows.
        "begin" => {
            let name = parse_environment_name(tokens, cursor);
            let Some((open, close)) = matrix_delimiters(&name) else {
                return Vec::new();
            };
            let rows = parse_matrix_rows(tokens, cursor);
            // The environment is closed by `\end{name}`, which is two
            // tokens: the command and the name. Both are read here, or the
            // name is left to be set as mathematics of its own.
            if matches!(tokens.get(*cursor), Some(Token::Command(name)) if name == "end") {
                *cursor += 1;
                let _ = parse_raw_text(tokens, cursor);
            }
            vec![Node::Matrix { open, close, rows }]
        }
        // The closing of an environment is consumed by the environment.
        "end" => Vec::new(),
        // A floor, a ceiling, and the modulo operator.
        "lfloor" | "lceil" | "langle" => {
            let symbol = match name {
                "lfloor" => "⌊",
                "lceil" => "⌈",
                _ => "⟨",
            };
            vec![Node::Text(symbol.to_string())]
        }
        "rfloor" | "rceil" | "rangle" => {
            let symbol = match name {
                "rfloor" => "⌋",
                "rceil" => "⌉",
                _ => "⟩",
            };
            vec![Node::Text(symbol.to_string())]
        }
        // A vertical bar names an absolute value or a norm. Written as a
        // command it is a delimiter like any other, and was being shown as
        // the letters of its own name.
        "lvert" | "rvert" | "vert" => {
            vec![attach_scripts(Node::Text("|".to_string()), tokens, cursor)]
        }
        "lVert" | "rVert" | "Vert" => {
            vec![attach_scripts(Node::Text("‖".to_string()), tokens, cursor)]
        }
        "lbrace" | "rbrace" => {
            vec![attach_scripts(Node::Text("{".to_string()), tokens, cursor)]
        }
        "bmod" => vec![Node::Text(" mod ".to_string())],
        "pmod" => {
            let argument = parse_argument(tokens, cursor);
            vec![Node::Row(vec![
                Node::Text(" (mod ".to_string()),
                argument,
                Node::Text(")".to_string()),
            ])]
        }
        // An accent, written as a mark above its argument.
        "dot" | "ddot" | "bar" | "overline" | "vec" | "hat" | "tilde" => {
            let mark = match name {
                "dot" => "˙",
                "ddot" => "¨",
                "vec" => "→",
                "hat" => "^",
                "tilde" => "~",
                _ => "‾",
            };
            let argument = parse_argument(tokens, cursor);
            let accented = Node::Overscript(Box::new(argument), mark.to_string());
            vec![attach_scripts(accented, tokens, cursor)]
        }
        // Commands that only set the size or the style, and carry no glyph.
        "thinspace" => vec![Node::Text("\u{2009}".to_string())],
        "medspace" | "thickspace" => vec![Node::Text("\u{2005}".to_string())],
        "negspace" => Vec::new(),
        "space" => vec![Node::Text(" ".to_string())],
        "quad" | "qquad" => vec![Node::Text("\u{2009}\u{2009}".to_string())],
        "displaystyle" | "textstyle" | "limits" | "nolimits" | "left" | "right" | "bigl"
        | "bigr" | "Bigl" | "Bigr" | "biggl" | "biggr" | "Biggl" | "Biggr" | "big" | "Big"
        | "bigg" | "Bigg" => {
            // A size or a pairing command is structural. The delimiter that
            // follows is read as an ordinary symbol.
            Vec::new()
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
        // A comma written as a command is punctuation, not a separator
        // between two formulas, so it keeps its own shape.
        "," => vec![Node::Text(", ".to_string())],
        // `\text{...}` holds words as they are written. Read as variables
        // they would be set one letter at a time, so `km` became `k m`.
        "text" | "mathrm" | "operatorname" | "mbox" => {
            let words = Node::Verbatim(parse_raw_text(tokens, cursor));
            vec![attach_scripts(words, tokens, cursor)]
        }
        // A style command keeps its argument set as mathematics. The group
        // it holds is a base, so it takes the scripts that follow it: the
        // `_central` of `\mathbf{a}_{\text{central}}` belongs to the `a`.
        "mathsf" | "mathtt" | "mathbf" | "mathit" | "mathcal" | "mathfrak" => {
            let inner = parse_argument(tokens, cursor);
            vec![attach_scripts(Node::Row(vec![inner]), tokens, cursor)]
        }
        // An unknown command is shown as written, so that a formula that
        // uses one still reads rather than losing part of itself.
        other => vec![Node::Text(format!("\\{other}"))],
    }
}

/// Writes a piece of text as the MathML element its kind calls for.
///
/// Every branch produces an element, and the last one produces `<mtext>`.
/// Text left bare inside a row is dropped by the browser, which loses part of
/// a formula without any sign that something is missing: a multi-digit number
/// such as the 4800 in `year + 4800` simply did not appear, and neither did
/// the name `delta_v` before it was given an element.
fn classify(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let first = text.chars().next().unwrap_or(' ');
    let all = |predicate: fn(char) -> bool| text.chars().all(predicate);

    // A space is a space. It is written as an explicit one rather than as
    // text, because a run of spaces between two elements carries no glyph.
    if all(char::is_whitespace) {
        return "<mspace width=\"0.25em\"/>".to_string();
    }
    // A variable is one letter, whatever script it is written in. Greek is
    // set as an identifier so that it is drawn as a variable rather than as
    // upright text.
    if text.chars().count() == 1 && first.is_alphabetic() {
        return format!("<mi>{}</mi>", escape(text));
    }

    // A number starts with a digit. It may hold a decimal point or an
    // exponent, as in `1.5` or `1e-3`.
    if first.is_ascii_digit()
        && all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '-' || c == '+')
    {
        return format!("<mn>{}</mn>", escape(text));
    }
    // A name of Latin letters is an identifier, whether it is one letter or
    // several, as in `a` or `delta_v`.
    if all(|c| c.is_ascii_alphabetic()) {
        return format!("<mi>{}</mi>", escape(text));
    }
    // An operator, including one written with its spacing, as in ` = `.
    // The spaces are the operator's own width, so they are trimmed before
    // the test rather than counted against it.
    let trimmed = text.trim();
    if !trimmed.is_empty() && trimmed.chars().all(|c| !c.is_alphanumeric()) {
        return format!("<mo>{}</mo>", escape(trimmed));
    }
    // Anything else is text: a label in another script, or a phrase. It is
    // set upright and kept whole rather than being read as variables.
    format!("<mtext>{}</mtext>", escape(text))
}

/// Writes a node as MathML.
fn emit(node: &Node) -> String {
    match node {
        // MathML distinguishes an identifier, a number and operators, and a
        // browser uses the element to choose the font: an upright digit in a
        // serif face reads as a number rather than as a variable.
        Node::Text(text) => classify(text),
        Node::Function(label) => format!("<mi>{}</mi>", escape(label)),
        Node::Verbatim(text) => format!("<mtext>{}</mtext>", escape(text)),
        Node::Overscript(base, mark) => format!(
            "<mover><mrow>{}</mrow><mo>{}</mo></mover>",
            emit(base),
            escape(mark)
        ),
        Node::Matrix { open, close, rows } => {
            let body: String = rows
                .iter()
                .map(|row| {
                    let cells: String = row
                        .iter()
                        .map(|cell| format!("<mtd>{}</mtd>", emit(cell)))
                        .collect();
                    format!("<mtr>{cells}</mtr>")
                })
                .collect();
            let table = format!("<mtable>{body}</mtable>");
            if open.is_empty() && close.is_empty() {
                table
            } else {
                format!("<mrow><mo>{open}</mo>{table}<mo>{close}</mo></mrow>")
            }
        }
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
