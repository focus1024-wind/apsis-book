//! A hand written JSON codec.
//!
//! The service must not depend on any crate other than `apsis`, so
//! encoding and decoding of JavaScript Object Notation (JSON) is implemented
//! here. The codec covers the subset of RFC 8259 that the service needs:
//!
//! - [`Value`] models any JSON value, including nested arrays and objects.
//! - [`parse`] decodes a document and reports a [`JsonError`] with the byte
//!   offset of the first problem.
//! - [`to_string`] encodes a value in compact form and [`to_string_pretty`]
//!   encodes it with indentation for readability.
//! - The [`json_object`](crate::json_object) macro builds an object literal
//!   without repeating the `Value` constructors.
//!
//! Numbers are held as `f64`. Integers that need the full `i64` range are
//! decoded exactly as long as they can be represented as an `f64` without
//! loss, which holds up to `2^53`; the service only exchanges quantities and
//! identifiers within that range.
//!
//! Encoding follows RFC 8259: strings are always quoted, control characters
//! and the quote and reverse solidus are escaped, and non-finite floats are
//! encoded as `null` because JSON has no representation for them.
//!
//! 手写的 JSON 编解码器。
//!
//! 服务除 `apsis` 外不得依赖任何 crate，因此 JSON 的编码与解码在此实现。
//! 该编解码器覆盖服务所需的 RFC 8259 子集：
//!
//! - [`Value`] 为任何 JSON 值建模，含嵌套的数组与对象。
//! - [`parse`] 解码一份文档，并以 [`JsonError`] 报告第一个问题的字节偏移。
//! - [`to_string`] 以紧凑形式编码一个值，[`to_string_pretty`] 以缩进编码以便
//!   阅读。
//! - [`json_object`](crate::json_object) 宏构造对象字面量，无需重复 `Value` 的构造函数。
//!
//! 数字以 `f64` 持有。需要完整 `i64` 范围的整数，只要能无损地表示为 `f64`，
//! 就会被精确解码，这在上限 `2^53` 之内成立；服务只交换该范围内的量与标识。
//!
//! 编码遵循 RFC 8259：字符串始终加引号，控制字符、引号与反斜杠被转义，非有限
//! 浮点数编码为 `null`，因为 JSON 没有它们的表示。

use std::fmt::{self, Write as _};

/// A JSON value.
///
/// 一个 JSON 值。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// The `null` literal.
    ///
    /// `null` 字面量。
    Null,
    /// A boolean, `true` or `false`.
    ///
    /// 一个布尔值，`true` 或 `false`。
    Bool(bool),
    /// A number. JSON does not distinguish integers from floats.
    ///
    /// 一个数值。JSON 不区分整数与浮点数。
    Number(f64),
    /// A string.
    ///
    /// 一个字符串。
    String(String),
    /// An array of values.
    ///
    /// 一个值的数组。
    Array(Vec<Value>),
    /// An object, preserving the order in which members were inserted.
    ///
    /// 一个对象，保持成员被插入时的顺序。
    Object(Vec<(String, Value)>),
}

impl Value {
    /// Creates a string value from anything that converts into a `String`.
    ///
    /// The constructor and [`Value::number`] complete the pair of scalar
    /// constructors for callers that build a value without a literal.
    ///
    /// 由任何可转换为 `String` 的东西创建一个字符串值。
    ///
    /// 该构造函数与 [`Value::number`] 构成一对标量构造函数，供不通过字面量构造取值
    /// 的调用方使用。
    #[allow(dead_code)]
    pub fn string(text: impl Into<String>) -> Self {
        Self::String(text.into())
    }

    /// Creates a number value.
    ///
    /// 创建一个数值。
    #[allow(dead_code)]
    pub fn number(value: impl Into<f64>) -> Self {
        Self::Number(value.into())
    }

    /// Returns the value as an object member list, or an empty slice when it
    /// is not an object.
    ///
    /// 把该值返回为对象的成员列表，若它不是对象则为空切片。
    pub fn members(&self) -> &[(String, Value)] {
        match self {
            Self::Object(members) => members,
            _ => &[],
        }
    }

    /// Returns the member with `key`, or `None` when it is absent or when
    /// the value is not an object.
    ///
    /// 返回键为 `key` 的成员；它不存在或该值不是对象时返回 `None`。
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.members()
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// Returns the value as a string slice, or `None` when it is not a
    /// string.
    ///
    /// 把该值返回为字符串切片，若它不是字符串则为 `None`。
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(text) => Some(text),
            _ => None,
        }
    }

    /// Returns the value as a number, or `None` when it is not a number.
    ///
    /// The accessor completes the set of typed reads on [`Value`]. Values
    /// that arrive from a request are read through the tool parameter
    /// wrappers, which validate a number against its declared bounds, so
    /// this is used where a value is known to be a number.
    ///
    /// 把该值返回为数值，若它不是数值则为 `None`。
    ///
    /// 该访问器补齐了 [`Value`] 上带类型的读取。来自请求的取值经由工具参数包装器
    /// 读取，那些包装器会依据声明的上下界校验数值；因此这里用在已知是数值的地方。
    #[allow(dead_code)]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the value as a `bool`, or `None` when it is not a boolean.
    ///
    /// 把该值返回为 `bool`，若它不是布尔值则为 `None`。
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the value as an array, or an empty slice when it is not an
    /// array.
    ///
    /// 把该值返回为数组，若它不是数组则为空切片。
    pub fn as_array(&self) -> &[Value] {
        match self {
            Self::Array(items) => items,
            _ => &[],
        }
    }
}

/// Builds an object [`Value`] from `key => value` pairs.
///
/// Each key is converted to a `String` and each value is converted to a
/// [`Value`], so literals such as `object!{"n" => 1.0}` work without
/// repeating the constructors.
///
/// 由 `key => value` 对构造一个对象 [`Value`]。
///
/// 每个键被转换为 `String`，每个值被转换为 [`Value`]，因此诸如
/// `object!{"n" => 1.0}` 的字面量无需重复构造函数即可使用。
#[macro_export]
macro_rules! json_object {
    () => { $crate::json::Value::Object(Vec::new()) };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        $crate::json::Value::Object(vec![
            $(
                (
                    ::std::string::String::from($key),
                    $crate::json::Value::from($value),
                ),
            )+
        ])
    }};
}

impl From<&str> for Value {
    fn from(text: &str) -> Self {
        Self::String(text.to_string())
    }
}

impl From<String> for Value {
    fn from(text: String) -> Self {
        Self::String(text)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Number(f64::from(value))
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Number(f64::from(value))
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Number(f64::from(value))
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Self::Number(value as f64)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(items: Vec<T>) -> Self {
        Self::Array(items.into_iter().map(Into::into).collect())
    }
}

/// A JSON decoding failure.
///
/// The offset is the byte position in the input where the problem was
/// detected, counted from zero. The message names the condition without
/// quoting the input, so that a malformed request cannot echo attacker
/// controlled text back through the error channel.
///
/// 一次 JSON 解码失败。
///
/// 偏移是从零数起的、输入中检测到问题的字节位置。消息指名具体条件而不复述输入，
/// 使畸形的请求无法经由错误通道回显攻击者控制的文本。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    /// Byte offset of the first problem.
    ///
    /// 第一个问题的字节偏移。
    pub offset: usize,
    /// Description of the problem.
    ///
    /// 问题的描述。
    pub message: String,
}

impl JsonError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
        }
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "json error at byte {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for JsonError {}

/// Decodes a JSON document.
///
/// Whitespace is allowed around every token. Trailing content after the
/// first complete value is rejected.
///
/// 解码一份 JSON 文档。
///
/// 每个记号周围都允许空白。第一个完整值之后的剩余内容会被拒绝。
/// Returns the length in bytes of the UTF-8 sequence starting with `byte`.
///
/// The leading byte states the length; the continuation bytes are checked
/// when the sequence itself is validated.
///
/// 返回以 `byte` 开头的 UTF-8 序列的字节长度。
///
/// 首字节给出长度；后续字节在该序列本身被校验时检查。
fn utf8_length(byte: u8) -> usize {
    match byte {
        0x00..=0x7f => 1,
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        // A continuation or an invalid leading byte; one byte is taken so
        // that the validation reports it as invalid rather than looping.

        // 一个后续字节或非法的首字节；只取一个字节，使校验把它报告为非法，
        // 而不是陷入循环。
        _ => 1,
    }
}

pub fn parse(input: &str) -> Result<Value, JsonError> {
    let mut parser = Parser {
        bytes: input.as_bytes(),
        position: 0,
    };
    parser.skip_whitespace();
    let value = parser.value()?;
    parser.skip_whitespace();
    if parser.position != parser.bytes.len() {
        return Err(JsonError::new(
            parser.position,
            "unexpected content after the top level value",
        ));
    }
    Ok(value)
}

/// Encodes a value in compact form.
///
/// 以紧凑形式编码一个值。
pub fn to_string(value: &Value) -> String {
    let mut output = String::new();
    // Writing into a String cannot fail, so the result is discarded.

    // 向 String 写入不会失败，因此结果被丢弃。
    let _ = write_value(&mut output, value);
    output
}

/// Encodes a value with two space indentation per nesting level.
///
/// A pretty document is larger than the compact form, so the service sends
/// the compact form; this is used when a payload is written for a reader.
///
/// 以每层嵌套两个空格的缩进编码一个值。
///
/// 带缩进的文档比紧凑形式更大，因此服务发送紧凑形式；这个用于为读者书写载荷时。
#[allow(dead_code)]
pub fn to_string_pretty(value: &Value) -> String {
    let mut output = String::new();
    let _ = write_pretty(&mut output, value, 0);
    output
}

/// Writes a value to `output` in compact form.
///
/// 以紧凑形式把一个值写入 `output`。
fn write_value(output: &mut String, value: &Value) -> fmt::Result {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(true) => output.push_str("true"),
        Value::Bool(false) => output.push_str("false"),
        Value::Number(number) => {
            write_number(output, *number)?;
        }
        Value::String(text) => {
            write_string(output, text)?;
        }
        Value::Array(items) => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_value(output, item)?;
            }
            output.push(']');
        }
        Value::Object(members) => {
            output.push('{');
            for (index, (key, member)) in members.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_string(output, key)?;
                output.push(':');
                write_value(output, member)?;
            }
            output.push('}');
        }
    }
    Ok(())
}

/// Writes a value to `output` with `depth` levels of indentation.
///
/// 以 `depth` 层缩进把一个值写入 `output`。
#[allow(dead_code)]
fn write_pretty(output: &mut String, value: &Value, depth: usize) -> fmt::Result {
    let indent = "  ".repeat(depth);
    let inner_indent = "  ".repeat(depth + 1);
    match value {
        Value::Array(items) if !items.is_empty() => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push('\n');
                output.push_str(&inner_indent);
                write_pretty(output, item, depth + 1)?;
            }
            output.push('\n');
            output.push_str(&indent);
            output.push(']');
        }
        Value::Object(members) if !members.is_empty() => {
            output.push('{');
            for (index, (key, member)) in members.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push('\n');
                output.push_str(&inner_indent);
                write_string(output, key)?;
                output.push_str(": ");
                write_pretty(output, member, depth + 1)?;
            }
            output.push('\n');
            output.push_str(&indent);
            output.push('}');
        }
        other => write_value(output, other)?,
    }
    Ok(())
}

/// Writes a number in a form JSON accepts.
///
/// Finite values are written with `f64`'s shortest round trip formatting.
/// Non-finite values have no JSON representation and are written as `null`,
/// so that a computation that produced a not-a-number value cannot produce
/// an unparsable document.
///
/// 以 JSON 接受的形式写出一个数值。
///
/// 有限值以 `f64` 的最短往返格式写出。非有限值没有 JSON 表示，写为 `null`，
/// 使产生了非数值的计算不可能产生一份无法解析的文档。
fn write_number(output: &mut String, number: f64) -> fmt::Result {
    if !number.is_finite() {
        output.push_str("null");
        return Ok(());
    }
    let _ = write!(output, "{number}");
    Ok(())
}

/// Writes a quoted string with the escapes required by RFC 8259.
///
/// Characters below `0x20` are escaped; the quote and reverse solidus are
/// escaped; every other character is emitted as is, which keeps UTF-8 input
/// valid because the input was already valid UTF-8.
///
/// 写出一个带 RFC 8259 所要求转义的加引号字符串。
///
/// `0x20` 以下的字符被转义；引号与反斜杠被转义；其余字符按原样输出，这保持了
/// UTF-8 输入的有效性，因为输入本就是合法 UTF-8。
fn write_string(output: &mut String, text: &str) -> fmt::Result {
    output.push('"');
    for character in text.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            control if control < '\u{20}' => {
                let _ = write!(output, "\\u{:04x}", control as u32);
            }
            other => output.push(other),
        }
    }
    output.push('"');
    Ok(())
}

/// A byte oriented recursive descent parser.
///
/// Recursion depth follows the nesting of the document; [`MAX_DEPTH`]
/// bounds it so that a hostile document cannot exhaust the stack.
///
/// 一个面向字节的递归下降解析器。
///
/// 递归深度跟随文档的嵌套；[`MAX_DEPTH`] 给它设界，使恶意文档无法耗尽栈。
struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
}

/// Maximum nesting depth accepted by [`parse`].
///
/// [`parse`] 接受的最大嵌套深度。
pub const MAX_DEPTH: usize = 64;

impl Parser<'_> {
    /// Parses a value at the current position.
    ///
    /// 在当前位置解析一个值。
    fn value(&mut self) -> Result<Value, JsonError> {
        self.value_at_depth(0)
    }

    /// Parses a value, rejecting nesting beyond [`MAX_DEPTH`].
    ///
    /// 解析一个值，拒绝超过 [`MAX_DEPTH`] 的嵌套。
    fn value_at_depth(&mut self, depth: usize) -> Result<Value, JsonError> {
        if depth > MAX_DEPTH {
            return Err(JsonError::new(self.position, "nesting is too deep"));
        }
        match self.peek() {
            Some(b'n') => self.literal("null", Value::Null),
            Some(b't') => self.literal("true", Value::Bool(true)),
            Some(b'f') => self.literal("false", Value::Bool(false)),
            Some(b'"') => self.string().map(Value::String),
            Some(b'[') => self.array(depth),
            Some(b'{') => self.object(depth),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => Err(JsonError::new(self.position, "unexpected token")),
            None => Err(JsonError::new(self.position, "unexpected end of input")),
        }
    }

    /// Returns the byte at the current position without consuming it.
    ///
    /// 返回当前位置的字节而不消费它。
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    /// Advances past any JSON whitespace.
    ///
    /// 越过任何 JSON 空白。
    fn skip_whitespace(&mut self) {
        while let Some(byte) = self.peek() {
            match byte {
                b' ' | b'\t' | b'\n' | b'\r' => self.position += 1,
                _ => break,
            }
        }
    }

    /// Consumes a fixed literal and returns `value`.
    ///
    /// 消费一个固定字面量并返回 `value`。
    fn literal(&mut self, text: &str, value: Value) -> Result<Value, JsonError> {
        if self.bytes[self.position..].starts_with(text.as_bytes()) {
            self.position += text.len();
            Ok(value)
        } else {
            Err(JsonError::new(self.position, "unexpected token"))
        }
    }

    /// Parses a quoted string, decoding the escape sequences.
    ///
    /// 解析一个加引号的字符串，并解码各转义序列。
    fn string(&mut self) -> Result<String, JsonError> {
        let start = self.position;
        self.position += 1; // opening quote
        let mut output = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(JsonError::new(start, "unterminated string"));
            };
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(output);
                }
                b'\\' => {
                    self.position += 1;
                    self.escape(&mut output)?;
                }
                control if control < 0x20 => {
                    return Err(JsonError::new(
                        self.position,
                        "unescaped control character in string",
                    ));
                }
                _ => {
                    // Decode one UTF-8 scalar from the bytes at this position.
                    //
                    // Only the sequence itself is validated. Validating the
                    // whole remaining buffer instead would be correct and
                    // quadratic, because it repeats for every character in
                    // the document: a catalogue of seven hundred kilobytes
                    // spent half a minute in this one line.

                    // 从此处的字节解码一个 UTF-8 标量。
                    //
                    // 只校验该序列本身。改为校验整个剩余缓冲虽然正确却是二次
                    // 复杂度，因为它会为文档中的每个字符重复一遍：一份七百
                    // 千字节的目录曾在这一行上花掉半分钟。
                    let length = utf8_length(byte);
                    let end = self.position + length;
                    if end > self.bytes.len() {
                        return Err(JsonError::new(self.position, "truncated utf-8 in string"));
                    }
                    let sequence = std::str::from_utf8(&self.bytes[self.position..end])
                        .map_err(|_| JsonError::new(self.position, "invalid utf-8 in string"))?;
                    output.push_str(sequence);
                    self.position = end;
                }
            }
        }
    }

    /// Decodes one escape sequence after the reverse solidus.
    ///
    /// 解码反斜杠之后的一个转义序列。
    fn escape(&mut self, output: &mut String) -> Result<(), JsonError> {
        let Some(byte) = self.peek() else {
            return Err(JsonError::new(self.position, "unterminated escape"));
        };
        self.position += 1;
        let character = match byte {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{08}',
            b'f' => '\u{0c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => return self.unicode_escape(output),
            _ => return Err(JsonError::new(self.position - 1, "unknown escape")),
        };
        output.push(character);
        Ok(())
    }

    /// Decodes a `\uXXXX` escape, pairing surrogates into one scalar value.
    ///
    /// 解码一个 `\uXXXX` 转义，把代理对合并为一个标量值。
    fn unicode_escape(&mut self, output: &mut String) -> Result<(), JsonError> {
        let first = self.hex4()?;
        let scalar = match first {
            // A high surrogate must be followed by a low surrogate.

            // 高代理之后必须跟一个低代理。
            0xd800..=0xdbff => {
                if self.peek() != Some(b'\\') {
                    return Err(JsonError::new(self.position, "unpaired high surrogate"));
                }
                self.position += 1;
                if self.peek() != Some(b'u') {
                    return Err(JsonError::new(self.position, "unpaired high surrogate"));
                }
                self.position += 1;
                let second = self.hex4()?;
                if !(0xdc00..=0xdfff).contains(&second) {
                    return Err(JsonError::new(self.position, "invalid surrogate pair"));
                }
                0x1_0000 + ((first - 0xd800) << 10) + (second - 0xdc00)
            }
            0xdc00..=0xdfff => {
                return Err(JsonError::new(self.position, "unpaired low surrogate"));
            }
            other => other,
        };
        let character = char::from_u32(scalar)
            .ok_or_else(|| JsonError::new(self.position, "escape is not a scalar value"))?;
        output.push(character);
        Ok(())
    }

    /// Reads exactly four hexadecimal digits.
    ///
    /// 恰好读取四个十六进制数字。
    fn hex4(&mut self) -> Result<u32, JsonError> {
        if self.position + 4 > self.bytes.len() {
            return Err(JsonError::new(self.position, "truncated unicode escape"));
        }
        let digits = &self.bytes[self.position..self.position + 4];
        let mut value = 0_u32;
        for digit in digits {
            let nibble = match digit {
                b'0'..=b'9' => u32::from(digit - b'0'),
                b'a'..=b'f' => u32::from(digit - b'a') + 10,
                b'A'..=b'F' => u32::from(digit - b'A') + 10,
                _ => return Err(JsonError::new(self.position, "invalid hexadecimal digit")),
            };
            value = (value << 4) | nibble;
        }
        self.position += 4;
        Ok(value)
    }

    /// Parses a number.
    ///
    /// The grammar is the one in RFC 8259: an optional minus sign, an
    /// integer part with no leading zero, an optional fraction, and an
    /// optional exponent. The token is collected in full and handed to the
    /// standard `f64` parser, which rounds once and applies the correct
    /// sign to every part of the value.
    ///
    /// 解析一个数值。
    ///
    /// 文法即 RFC 8259 的那一个：可选的负号、无前导零的整数部分、可选的小数部分、
    /// 可选的指数。该记号被完整收集后交给标准库的 `f64` 解析器，它只做一次舍入，
    /// 并对该值的每一部分施加正确的符号。
    fn number(&mut self) -> Result<Value, JsonError> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        let integer_start = self.position;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.position += 1;
        }
        if self.position == integer_start {
            return Err(JsonError::new(start, "number has no digits"));
        }

        if self.peek() == Some(b'.') {
            self.position += 1;
            let fraction_start = self.position;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if self.position == fraction_start {
                return Err(JsonError::new(self.position, "fraction has no digits"));
            }
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.position += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.position += 1;
            }
            let exponent_start = self.position;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if self.position == exponent_start {
                return Err(JsonError::new(self.position, "exponent has no digits"));
            }
        }

        let text = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| JsonError::new(start, "number is not valid text"))?;
        let value: f64 = text
            .parse()
            .map_err(|_| JsonError::new(start, "number is out of range"))?;
        if !value.is_finite() {
            return Err(JsonError::new(start, "number is out of range"));
        }
        Ok(Value::Number(value))
    }

    /// Parses an array.
    ///
    /// 解析一个数组。
    fn array(&mut self, depth: usize) -> Result<Value, JsonError> {
        self.position += 1; // opening bracket
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(Value::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value_at_depth(depth + 1)?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    return Ok(Value::Array(items));
                }
                _ => return Err(JsonError::new(self.position, "expected ',' or ']'")),
            }
        }
    }

    /// Parses an object.
    ///
    /// 解析一个对象。
    fn object(&mut self, depth: usize) -> Result<Value, JsonError> {
        self.position += 1; // opening brace
        let mut members = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(Value::Object(members));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err(JsonError::new(self.position, "expected a member name"));
            }
            let key = self.string()?;
            self.skip_whitespace();
            if self.peek() != Some(b':') {
                return Err(JsonError::new(self.position, "expected ':'"));
            }
            self.position += 1;
            self.skip_whitespace();
            let value = self.value_at_depth(depth + 1)?;
            members.push((key, value));
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    return Ok(Value::Object(members));
                }
                _ => return Err(JsonError::new(self.position, "expected ',' or '}'")),
            }
        }
    }
}
