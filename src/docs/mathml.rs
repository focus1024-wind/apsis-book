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
//!
//! 为文档渲染数学记法。
//!
//! 文档带有公式，读者需要它们看起来像公式而不是像源码。完整的 TeX 引擎是一个
//! 庞大的依赖并自带字体文件，因此这里把记法翻译为 MathML。浏览器自行排版与打印
//! MathML，因此无需运行任何脚本，输出在没有 JavaScript 时也可用。
//!
//! 所覆盖的子集正是轨道力学实际使用的那些：分式、上标、下标、根式、希腊字母、
//! 常见运算符，以及用于分组它们的定界符。子集之外的表达式按原样显示而不是丢弃，
//! 因此一个不被支持的命令不会有任何损失，只是显得朴素。
//!
//! 识别两种形式，与文档所用的约定一致：句内公式用 `$...$`，独占一行用
//! `$$...$$`。

use crate::docs::markdown::{escape, escape_attr};

/// Converts a formula to MathML wrapped in a container.
///
/// `display` is set when the formula stands on its own line, which makes it
/// larger and centred.
///
/// 把公式转换为包在容器中的 MathML。
///
/// 当公式独占一行时置上 `display`，这使它更大并居中。
pub fn render(latex: &str, display: bool) -> String {
    // The delimiters of a display formula are on their own lines, so the
    // source arrives with the newline that followed the opening pair.

    // 独占一行公式的定界符各占一行，因此源码到手时带着开头那一对之后的换行。
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

    // 原始源码保存在 data 属性中，使公式可以从页面恢复，供复制与测试使用。
    // MathML 是一个 token 一个元素，而浏览器复制选区时会逐个走过它们：从页面
    // 复制一条公式会得到每行一个字符。因此源码被携带在其旁一个读者可选的元素里，
    // 而 MathML 被移出选区，使复制得到的是公式书写时的样子。
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
///
/// 把源码改写为翻译循环所读的记号。
///
/// 花括号被变成单字符，使一个组成为一个单位；空格类命令被丢弃。
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

                // 命令一直延伸到第一个非字母字符。
                let mut end = index + 1;
                while end < characters.len() && characters[end].is_ascii_alphabetic() {
                    end += 1;
                }
                if end == index + 1 {
                    // A backslash before a mark is a command of its own when
                    // the mark is a spacing one, and that character itself
                    // otherwise.

                    // 反斜杠后若是空格类记号，它自成一个命令；否则那个字符本身。
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
///
/// 归一化之后公式中的一个单位。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    /// A single character.

    ///
    /// 单个字符。
    Symbol(char),
    /// A backslash command, without the backslash.

    ///
    /// 一个反斜杠命令，不含反斜杠。
    Command(String),
    /// An opening or closing brace; which one is taken from the position.

    ///
    /// 一个开括号或闭括号；具体是哪一个由位置决定。
    Group,
}

/// A parsed expression, ready to become MathML.
///
/// 一个已解析的表达式，可以变为 MathML。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    /// Text, already escaped.

    ///
    /// 文本，已转义。
    Text(String),
    /// A row of children.

    ///
    /// 一行子节点。
    Row(Vec<Node>),
    /// A fraction.

    ///
    /// 一个分式。
    Fraction(Box<Node>, Box<Node>),
    /// A root.

    ///
    /// 一个根式。
    Root(Box<Node>, Option<Box<Node>>),
    /// A superscript.

    ///
    /// 一个上标。
    Superscript(Box<Node>, Box<Node>),
    /// A subscript.

    ///
    /// 一个下标。
    Subscript(Box<Node>, Box<Node>),
    /// A subscript and a superscript on one base.

    ///
    /// 同一个基上的下标与上标。
    Scripts(Box<Node>, Box<Node>, Box<Node>),
    /// A named function such as `sin`.

    ///
    /// 一个具名函数，例如 `sin`。
    Function(String),
    /// A mark written above its argument, as in `\dot\Omega`.

    ///
    /// 写在参数上方的一个符号，例如 `\dot\Omega`。
    Overscript(Box<Node>, String),
    /// Words that stand as they are written, as in `\text{km}`.

    ///
    /// 按书写原样保留的词，例如 `\text{km}`。
    Verbatim(String),
    /// A matrix, with the delimiters its environment asks for.

    ///
    /// 一个矩阵，带其环境所要求的定界符。
    Matrix {
        /// The opening and closing delimiters, which may be empty.

        ///
        /// 开定界符与闭定界符，可以为空。
        open: &'static str,
        close: &'static str,
        /// The cells, one row at a time.

        ///
        /// 各单元格，一次一行。
        rows: Vec<Vec<Node>>,
    },
}

/// The Greek letters, by command.
///
/// 各希腊字母，按命令查。
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
///
/// 各运算符，按命令查。
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
///
/// 各函数，按命令查；它们被排成直立体而不是斜体。
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
///
/// 把一串记号渲染为 MathML。
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
///
/// 一直解析到输入结束，或在给出 `stop` 时解析到结束外层组的那个记号。
fn parse_row(tokens: &[Token], cursor: &mut usize, stop: Option<&[char]>) -> Vec<Node> {
    let mut row: Vec<Node> = Vec::new();

    while *cursor < tokens.len() {
        match &tokens[*cursor] {
            // A closing brace ends the enclosing group; a delimiter set
            // ends the stretchy group.

            // 闭括号结束外层组；一个定界符集合结束可变高的组。
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
///
/// 返回某个矩阵环境绘制所用的定界符。
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
///
/// 从 `\begin` 之后的组中读出环境名。
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
///
/// 读取矩阵的各行，直到闭合它的 `\end`。
///
/// 单元格在 `&` 处结束，一行在 `\\` 处结束，环境在 `\end` 处结束。单元格的
/// 记号由与其他地方相同的处理函数读取，因此单元格内的分式或根式与在外面的表现
/// 一致。
fn parse_matrix_rows(tokens: &[Token], cursor: &mut usize) -> Vec<Vec<Node>> {
    // Two accumulators: the cell being read, and the cells of the row so
    // far. One alone does not do, because `&` ends a cell and starts the
    // next, and the nodes after it belong to the next cell rather than
    // beside the one just closed.

    // 两个累加器：正在读取的单元格，以及本行此前的各单元格。只用一个不够，
    // 因为 `&` 结束一个单元格并开启下一个，其后的节点属于下一个单元格，而不是
    // 与刚关闭的那个并列。
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

    // 最后一行之后没有 `\\`。
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
///
/// 把参数的字符按其书写原样读取为文本。
///
/// 用于参数是词而不是数学的地方，使这些字母不会逐字被读成变量。
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
///
/// 读取命令的参数，它或是一个组，或是一个符号。
fn parse_argument(tokens: &[Token], cursor: &mut usize) -> Node {
    match tokens.get(*cursor) {
        Some(Token::Group) => {
            *cursor += 1;
            Node::Row(parse_row(tokens, cursor, Some(&['}'])))
        }
        // A square bracket is a group too, and is how a root takes its
        // index: `\sqrt[3]{x}`.

        // 方括号也是一个组，根式正是用它取指数：`\sqrt[3]{x}`。
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
///
/// 读取跟随某个基的上下标，并返回带上它们的节点。
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
///
/// 某个位置上的数字。
fn characters_digit(tokens: &[Token], cursor: usize) -> char {
    match tokens.get(cursor) {
        Some(Token::Symbol(symbol)) => *symbol,
        _ => '0',
    }
}

/// Handles a plain character, including the brackets that group.
///
/// 处理一个普通字符，包括用于分组的括号。
fn parse_symbol(symbol: char, tokens: &[Token], cursor: &mut usize, _depth: &mut u32) -> Vec<Node> {
    let base = match symbol {
        // Digits are read together as one number, so that `12` is a number
        // rather than two identifiers. A dot between digits belongs to it,
        // which also stops the layout breaking the number across a line.

        // 数字被一起读成一个数，使 `12` 是一个数而不是两个标识符。数字之间的
        // 点属于它，这也阻止排版把这个数断成两行。
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

        // 开括号会伸展到其内容的高度。
        //
        // 括号组是一个与其他基无异的基，因此由本函数末尾的同一处调用给它挂接
        // 上下标。若从这里直接返回该组，`(a + b)^2` 会把脱字符当作普通符号排版、
        // 指数落在其旁，而在分式内指数会整个逃出分式。
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

        // 空格在词与词之间被保留、在其他地方被丢弃，这使 `a b` 分开，而括号之后
        // 不留空隙。
        other => Node::Text(other.to_string()),
    };
    vec![attach_scripts(base, tokens, cursor)]
}

/// Handles a backslash command.
///
/// 处理一个反斜杠命令。
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

        // 函数是直立体，因此它作为自己的节点输出，而不是作为一个斜体变量。
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

        // 矩阵环境：\begin{name} 各单元格 \end{name}，行内单元格之间用 `&`，
        // 各行之间用 `\\`。
        "begin" => {
            let name = parse_environment_name(tokens, cursor);
            let Some((open, close)) = matrix_delimiters(&name) else {
                return Vec::new();
            };
            let rows = parse_matrix_rows(tokens, cursor);
            // The environment is closed by `\end{name}`, which is two
            // tokens: the command and the name. Both are read here, or the
            // name is left to be set as mathematics of its own.

            // 环境由 `\end{name}` 闭合，那是两个记号：命令与名字。两者都在此
            // 读取，否则名字会被留下、被当作数学单独排版。
            if matches!(tokens.get(*cursor), Some(Token::Command(name)) if name == "end") {
                *cursor += 1;
                let _ = parse_raw_text(tokens, cursor);
            }
            vec![Node::Matrix { open, close, rows }]
        }
        // The closing of an environment is consumed by the environment.

        // 环境的闭合由环境本身消费。
        "end" => Vec::new(),
        // A floor, a ceiling, and the modulo operator.

        // 下取整、上取整，以及取模运算符。
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

        // 竖线表示绝对值或范数。写成命令时它与其他定界符无异，而此前它被显示成
        // 自己的名字那些字母。
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

        // 一个重音符号，写在参数上方。
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

        // 只设定大小或样式、不携带字形的命令。
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

            // 大小命令或配对命令是结构性的。其后的定界符按普通符号读取。
            Vec::new()
        }
        "sqrt" => {
            // The index, when there is one, is given in brackets before the
            // radicand: `\sqrt[3]{x}`.

            // 指数（若有）写在被开方式之前、方括号之中：`\sqrt[3]{x}`。
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

        // 写成命令的逗号是标点，而不是两条公式之间的分隔符，因此它保留自己的
        // 形状。
        "," => vec![Node::Text(", ".to_string())],
        // `\text{...}` holds words as they are written. Read as variables
        // they would be set one letter at a time, so `km` became `k m`.

        // `\text{...}` 按书写原样保留词。若当作变量读取，它们会被逐字母排版，
        // 于是 `km` 变成了 `k m`。
        "text" | "mathrm" | "operatorname" | "mbox" => {
            let words = Node::Verbatim(parse_raw_text(tokens, cursor));
            vec![attach_scripts(words, tokens, cursor)]
        }
        // A style command keeps its argument set as mathematics. The group
        // it holds is a base, so it takes the scripts that follow it: the
        // `_central` of `\mathbf{a}_{\text{central}}` belongs to the `a`.

        // 样式命令让其参数仍按数学排版。它持有的组是一个基，因此接收其后的
        // 上下标：`\mathbf{a}_{\text{central}}` 中的 `_central` 属于那个 `a`。
        "mathsf" | "mathtt" | "mathbf" | "mathit" | "mathcal" | "mathfrak" => {
            let inner = parse_argument(tokens, cursor);
            vec![attach_scripts(Node::Row(vec![inner]), tokens, cursor)]
        }
        // An unknown command is shown as written, so that a formula that
        // uses one still reads rather than losing part of itself.

        // 未知命令按原样显示，使使用了它的公式仍然可读，而不是丢掉自己的一部分。
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
///
/// 把一段文本按其种类所要求的 MathML 元素写出。
///
/// 每个分支都产出一个元素，最后一个产出 `<mtext>`。留在行内没有元素的文本会被
/// 浏览器丢弃，这会丢掉公式的一部分而没有任何缺失的迹象：`year + 4800` 里的
/// 4800 就完全没有出现，而在被赋予元素之前，名字 `delta_v` 也是如此。
fn classify(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let first = text.chars().next().unwrap_or(' ');
    let all = |predicate: fn(char) -> bool| text.chars().all(predicate);

    // A space is a space. It is written as an explicit one rather than as
    // text, because a run of spaces between two elements carries no glyph.

    // 空格就是空格。它被写成显式的空格而不是文本，因为两个元素之间的一串空格
    // 不携带任何字形。
    if all(char::is_whitespace) {
        return "<mspace width=\"0.25em\"/>".to_string();
    }
    // A variable is one letter, whatever script it is written in. Greek is
    // set as an identifier so that it is drawn as a variable rather than as
    // upright text.

    // 变量是一个字母，无论用哪种文字书写。希腊字母被排成标识符，使它被绘制为
    // 变量而不是直立体文本。
    if text.chars().count() == 1 && first.is_alphabetic() {
        return format!("<mi>{}</mi>", escape(text));
    }

    // A number starts with a digit. It may hold a decimal point or an
    // exponent, as in `1.5` or `1e-3`.

    // 数字以十进制数字开头。它可以含小数点或指数，例如 `1.5` 或 `1e-3`。
    if first.is_ascii_digit()
        && all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '-' || c == '+')
    {
        return format!("<mn>{}</mn>", escape(text));
    }
    // A name of Latin letters is an identifier, whether it is one letter or
    // several, as in `a` or `delta_v`.

    // 拉丁字母构成的名字是标识符，无论是一个字母还是多个，例如 `a` 或
    // `delta_v`。
    if all(|c| c.is_ascii_alphabetic()) {
        return format!("<mi>{}</mi>", escape(text));
    }
    // An operator, including one written with its spacing, as in ` = `.
    // The spaces are the operator's own width, so they are trimmed before
    // the test rather than counted against it.

    // 一个运算符，包括带空格书写的那些，例如 ` = `。空格是运算符自身的宽度，
    // 因此在判断之前先去掉它们，而不是把它们算作否定条件。
    let trimmed = text.trim();
    if !trimmed.is_empty() && trimmed.chars().all(|c| !c.is_alphanumeric()) {
        return format!("<mo>{}</mo>", escape(trimmed));
    }
    // Anything else is text: a label in another script, or a phrase. It is
    // set upright and kept whole rather than being read as variables.

    // 其余一切都是文本：另一种文字写成的标签，或一个短语。它被排成直立体并保持
    // 完整，而不是被读成变量。
    format!("<mtext>{}</mtext>", escape(text))
}

/// Writes a node as MathML.
///
/// 把一个节点写为 MathML。
fn emit(node: &Node) -> String {
    match node {
        // MathML distinguishes an identifier, a number and operators, and a
        // browser uses the element to choose the font: an upright digit in a
        // serif face reads as a number rather than as a variable.

        // MathML 区分标识符、数字与运算符，浏览器用元素来选择字体：衬线体中的
        // 直立数字读起来是数字而不是变量。
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

            // 只含一个子节点的一行就是该子节点本身；多一对括号只会成为标记中的
            // 噪声，并破坏元素布局所依赖的分组。
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
