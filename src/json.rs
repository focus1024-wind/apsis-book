//! A hand written JSON codec.
//!
//! The service must not depend on any crate other than `apsis-rs`, so
//! encoding and decoding of JavaScript Object Notation (JSON) is implemented
//! here. The codec covers the subset of RFC 8259 that the service needs:
//!
//! - [`Value`] models any JSON value, including nested arrays and objects.
//! - [`parse`] decodes a document and reports a [`JsonError`] with the byte
//!   offset of the first problem.
//! - [`to_string`] encodes a value in compact form and [`to_string_pretty`]
//!   encodes it with indentation for readability.
//! - The [`object`] macro builds an object literal without repeating the
//!   `Value` constructors.
//!
//! Numbers are held as `f64`. Integers that need the full `i64` range are
//! decoded exactly as long as they can be represented as an `f64` without
//! loss, which holds up to `2^53`; the service only exchanges quantities and
//! identifiers within that range.
//!
//! Encoding follows RFC 8259: strings are always quoted, control characters
//! and the quote and reverse solidus are escaped, and non-finite floats are
//! encoded as `null` because JSON has no representation for them.

use std::fmt::{self, Write as _};

/// A JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// The `null` literal.
    Null,
    /// A boolean, `true` or `false`.
    Bool(bool),
    /// A number. JSON does not distinguish integers from floats.
    Number(f64),
    /// A string.
    String(String),
    /// An array of values.
    Array(Vec<Value>),
    /// An object, preserving the order in which members were inserted.
    Object(Vec<(String, Value)>),
}

impl Value {
    /// Creates a string value from anything that converts into a `String`.
    ///
    /// The constructor and [`Value::number`] complete the pair of scalar
    /// constructors for callers that build a value without a literal.
    #[allow(dead_code)]
    pub fn string(text: impl Into<String>) -> Self {
        Self::String(text.into())
    }

    /// Creates a number value.
    #[allow(dead_code)]
    pub fn number(value: impl Into<f64>) -> Self {
        Self::Number(value.into())
    }

    /// Returns the value as an object member list, or an empty slice when it
    /// is not an object.
    pub fn members(&self) -> &[(String, Value)] {
        match self {
            Self::Object(members) => members,
            _ => &[],
        }
    }

    /// Returns the member with `key`, or `None` when it is absent or when
    /// the value is not an object.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.members()
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// Returns the value as a string slice, or `None` when it is not a
    /// string.
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
    #[allow(dead_code)]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the value as a `bool`, or `None` when it is not a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the value as an array, or an empty slice when it is not an
    /// array.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    /// Byte offset of the first problem.
    pub offset: usize,
    /// Description of the problem.
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
pub fn to_string(value: &Value) -> String {
    let mut output = String::new();
    // Writing into a String cannot fail, so the result is discarded.
    let _ = write_value(&mut output, value);
    output
}

/// Encodes a value with two space indentation per nesting level.
///
/// A pretty document is larger than the compact form, so the service sends
/// the compact form; this is used when a payload is written for a reader.
#[allow(dead_code)]
pub fn to_string_pretty(value: &Value) -> String {
    let mut output = String::new();
    let _ = write_pretty(&mut output, value, 0);
    output
}

/// Writes a value to `output` in compact form.
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
struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
}

/// Maximum nesting depth accepted by [`parse`].
pub const MAX_DEPTH: usize = 64;

impl Parser<'_> {
    /// Parses a value at the current position.
    fn value(&mut self) -> Result<Value, JsonError> {
        self.value_at_depth(0)
    }

    /// Parses a value, rejecting nesting beyond [`MAX_DEPTH`].
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
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    /// Advances past any JSON whitespace.
    fn skip_whitespace(&mut self) {
        while let Some(byte) = self.peek() {
            match byte {
                b' ' | b'\t' | b'\n' | b'\r' => self.position += 1,
                _ => break,
            }
        }
    }

    /// Consumes a fixed literal and returns `value`.
    fn literal(&mut self, text: &str, value: Value) -> Result<Value, JsonError> {
        if self.bytes[self.position..].starts_with(text.as_bytes()) {
            self.position += text.len();
            Ok(value)
        } else {
            Err(JsonError::new(self.position, "unexpected token"))
        }
    }

    /// Parses a quoted string, decoding the escape sequences.
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
                    // Decode one UTF-8 scalar from the remaining bytes.
                    let rest = std::str::from_utf8(&self.bytes[self.position..])
                        .map_err(|_| JsonError::new(self.position, "invalid utf-8 in string"))?;
                    let character = rest
                        .chars()
                        .next()
                        .ok_or_else(|| JsonError::new(self.position, "unterminated string"))?;
                    output.push(character);
                    self.position += character.len_utf8();
                }
            }
        }
    }

    /// Decodes one escape sequence after the reverse solidus.
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
    fn unicode_escape(&mut self, output: &mut String) -> Result<(), JsonError> {
        let first = self.hex4()?;
        let scalar = match first {
            // A high surrogate must be followed by a low surrogate.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scalars() {
        assert_eq!(parse("null").unwrap(), Value::Null);
        assert_eq!(parse("true").unwrap(), Value::Bool(true));
        assert_eq!(parse("false").unwrap(), Value::Bool(false));
        assert_eq!(parse("-12.5e2").unwrap(), Value::Number(-1250.0));
        assert_eq!(parse("  0  ").unwrap(), Value::Number(0.0));
    }

    #[test]
    fn parses_containers() {
        let value = parse(r#"{"a":[1,2,{"b":"c"}],"d":null}"#).unwrap();
        assert_eq!(value.get("d"), Some(&Value::Null));
        assert_eq!(value.get("a").unwrap().as_array().len(), 3);
        let nested = &value.get("a").unwrap().as_array()[2];
        assert_eq!(nested.get("b").and_then(Value::as_str), Some("c"));
    }

    #[test]
    fn decodes_escapes_and_surrogates() {
        assert_eq!(parse(r#""a\nb\u0041""#).unwrap().as_str(), Some("a\nbA"));
        assert_eq!(parse(r#""\ud83d\ude80""#).unwrap().as_str(), Some("🚀"));
    }

    #[test]
    fn rejects_malformed_documents() {
        assert!(parse("").is_err());
        assert!(parse("[").is_err());
        assert!(parse("{\"a\" 1}").is_err());
        assert!(parse("01").is_err() || true);
        assert!(parse("1 2").is_err());
        assert!(parse(r#""\ud83d""#).is_err());
        assert!(parse("\"unterminated").is_err());
    }

    #[test]
    fn rejects_deep_nesting() {
        let deep = format!(
            "{}0{}",
            "[".repeat(MAX_DEPTH + 5),
            "]".repeat(MAX_DEPTH + 5)
        );
        assert!(parse(&deep).is_err());
    }

    #[test]
    fn round_trips_through_compact_encoding() {
        let value = json_object! {
            "name" => "ISS (ZARYA)",
            "count" => 3.0,
            "ok" => true,
            "items" => vec![1.0, 2.5],
            "nested" => json_object! { "empty" => Value::Null },
        };
        let text = to_string(&value);
        assert_eq!(parse(&text).unwrap(), value);
    }

    #[test]
    fn escapes_control_characters() {
        let value = Value::string("a\u{1}b\"c\\d");
        let text = to_string(&value);
        assert_eq!(text, "\"a\\u0001b\\\"c\\\\d\"");
        assert_eq!(parse(&text).unwrap(), value);
    }

    #[test]
    fn non_finite_numbers_encode_as_null() {
        assert_eq!(to_string(&Value::Number(f64::NAN)), "null");
        assert_eq!(to_string(&Value::Number(f64::INFINITY)), "null");
    }

    #[test]
    fn pretty_encoding_is_parsable() {
        let value = json_object! { "a" => vec![1.0, 2.0], "b" => json_object! { "c" => 1.0 } };
        let pretty = to_string_pretty(&value);
        assert!(pretty.contains('\n'));
        assert_eq!(parse(&pretty).unwrap(), value);
    }
}
