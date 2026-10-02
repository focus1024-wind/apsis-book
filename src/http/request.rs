//! Parsing of inbound HTTP/1.1 requests.
//!
//! 入站 HTTP/1.1 请求的解析。

use std::collections::HashMap;
use std::fmt;
use std::io::{self, Read};

/// The request method.
///
/// Only the methods the service uses are distinguished. Any other token is
/// kept as [`Method::Other`] so that it can be rejected by the router with
/// an accurate status rather than failing during parsing.
///
/// 请求方法。
///
/// 只区分服务所用的方法。其余任何记号都保留为 [`Method::Other`]，以便由路由器
/// 以准确的状态码拒绝，而不是在解析阶段失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    /// `GET`.
    /// `GET`。
    Get,
    /// `POST`.
    /// `POST`。
    Post,
    /// `HEAD`.
    /// `HEAD`。
    Head,
    /// `OPTIONS`.
    /// `OPTIONS`。
    Options,
    /// Any other method token.
    /// 任何其他方法记号。
    Other(String),
}

impl Method {
    /// Parses a method token.
    /// 解析一个方法记号。
    pub fn parse(token: &str) -> Self {
        match token {
            "GET" => Self::Get,
            "POST" => Self::Post,
            "HEAD" => Self::Head,
            "OPTIONS" => Self::Options,
            other => Self::Other(other.to_string()),
        }
    }

    /// Returns the method as it is written on the wire.
    /// 以线路上书写的原样返回该方法。
    pub fn as_str(&self) -> &str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
            Self::Other(token) => token.as_str(),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A parsed request.
/// 一个已解析的请求。
#[derive(Debug, Clone)]
pub struct Request {
    /// Request method.
    /// 请求方法。
    pub method: Method,
    /// Request target, exactly as received, without the query component.
    /// 请求目标，按收到时的原样，不含查询部分。
    pub path: String,
    /// Query component, without the leading question mark, if one was sent.
    /// 查询部分，不含前导问号，若发送过。
    pub query: Option<String>,
    /// Header fields, with names lowercased for case insensitive lookup.
    /// 首部字段，名字已转为小写以便不区分大小写地查找。
    pub headers: HashMap<String, String>,
    /// Request body.
    /// 请求体。
    pub body: Vec<u8>,
}

impl Request {
    /// Returns the value of a header field, matching the name case
    /// insensitively.
    ///
    /// The parser populates the header map for every request, so a route
    /// that needs a header, such as `Accept` or `Content-Type`, reads it
    /// here.
    ///
    /// 返回某个首部字段的值，名字匹配不区分大小写。
    ///
    /// 解析器为每个请求填充首部表，因此需要某个首部的路由——例如 `Accept` 或
    /// `Content-Type`——从这里读取它。
    #[allow(dead_code)]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    /// Returns the body decoded as UTF-8.
    ///
    /// Returns `None` when the body is not valid UTF-8; callers that accept
    /// textual bodies report that as a client error.
    ///
    /// 返回按 UTF-8 解码后的请求体。
    ///
    /// 请求体不是合法 UTF-8 时返回 `None`；接受文本体的调用方把它报告为客户端
    /// 错误。
    pub fn body_text(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }

    /// Returns the query string as a map of decoded key and value pairs.
    ///
    /// Pairs are separated by `&`, keys and values by `=`, and `+` decodes
    /// to a space. Percent escapes are decoded; a malformed escape is kept
    /// literally so that parsing never fails.
    ///
    /// 把查询串返回为已解码的键值对映射。
    ///
    /// 各对之间以 `&` 分隔，键与值之间以 `=` 分隔，`+` 解码为空格。百分号转义
    /// 会被解码；畸形的转义按字面保留，使解析永不失败。
    pub fn query_pairs(&self) -> HashMap<String, String> {
        let mut pairs = HashMap::new();
        let Some(query) = &self.query else {
            return pairs;
        };
        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (key, value) = match pair.split_once('=') {
                Some((key, value)) => (key, value),
                None => (pair, ""),
            };
            pairs.insert(percent_decode(key), percent_decode(value));
        }
        pairs
    }
}

/// Decodes `+` and percent escapes in a query component.
///
/// 解码查询部分中的 `+` 与百分号转义。
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                match (hex(bytes[index + 1]), hex(bytes[index + 2])) {
                    (Some(high), Some(low)) => {
                        output.push((high << 4) | low);
                        index += 3;
                    }
                    _ => {
                        output.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(output)
        .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned())
}

/// Returns the value of a hexadecimal digit.
///
/// 返回一个十六进制数字的值。
fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// A request parsing failure.
///
/// 一次请求解析失败。
#[derive(Debug)]
pub enum RequestError {
    /// The connection failed before a complete request was read.
    ///
    /// 在读完一个完整请求之前连接就失败了。
    Io(io::Error),
    /// The request line, a header line, or the body was malformed.
    ///
    /// 请求行、某个首部行或请求体畸形。
    Malformed(&'static str),
    /// The request exceeds the configured size limits.
    ///
    /// 请求超出配置的大小上限。
    TooLarge(&'static str),
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "connection error: {error}"),
            Self::Malformed(detail) => write!(f, "malformed request: {detail}"),
            Self::TooLarge(detail) => write!(f, "request too large: {detail}"),
        }
    }
}

impl std::error::Error for RequestError {}

impl From<io::Error> for RequestError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Limits applied while reading a request.
///
/// 读取请求时施加的上限。
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Maximum number of header lines.
    /// 首部行的最大数量。
    pub max_headers: usize,
    /// Maximum size of the head, meaning the request line and the headers.
    /// 头部的最大大小，即请求行与各首部之和。
    pub max_head_bytes: usize,
    /// Maximum size of the body.
    /// 请求体的最大大小。
    pub max_body_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_headers: 64,
            max_head_bytes: 16 * 1024,
            max_body_bytes: 1024 * 1024,
        }
    }
}

/// Reads and parses one request from `stream`.
///
/// The head is read one byte at a time so that any body bytes that follow it
/// stay in the stream. The function returns when a complete request has been
/// read or when the stream ends.
///
/// 从 `stream` 读取并解析一个请求。
///
/// 头部逐字节读取，使紧随其后的请求体字节留在流中。函数在读完一个完整请求或流
/// 结束时返回。
pub fn read_request<R: Read>(stream: &mut R, limits: Limits) -> Result<Request, RequestError> {
    let head = read_head(stream, limits)?;
    let head = std::str::from_utf8(&head)
        .map_err(|_| RequestError::Malformed("the request head is not valid utf-8"))?;

    let mut lines = head.split("\r\n");
    let request_line = lines
        .next()
        .ok_or(RequestError::Malformed("missing request line"))?;
    let mut parts = request_line.split(' ');
    let method = parts
        .next()
        .filter(|token| !token.is_empty())
        .ok_or(RequestError::Malformed("missing method"))?;
    let target = parts
        .next()
        .filter(|token| !token.is_empty())
        .ok_or(RequestError::Malformed("missing request target"))?;
    let version = parts
        .next()
        .ok_or(RequestError::Malformed("missing protocol version"))?;
    if parts.next().is_some() {
        return Err(RequestError::Malformed("trailing text in the request line"));
    }
    if !version.starts_with("HTTP/1.") {
        return Err(RequestError::Malformed("unsupported protocol version"));
    }

    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or(RequestError::Malformed("header line has no colon"))?;
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty() {
            return Err(RequestError::Malformed("header name is empty"));
        }
        headers
            .entry(name)
            .or_insert_with(|| value.trim().to_string());
    }

    let (path, query) = split_target(target);
    let length = match headers.get("content-length") {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| RequestError::Malformed("content-length is not a number"))?,
        None => 0,
    };
    if length > limits.max_body_bytes {
        return Err(RequestError::TooLarge("body exceeds the size limit"));
    }

    let mut body = vec![0_u8; length];
    stream.read_exact(&mut body)?;

    Ok(Request {
        method: Method::parse(method),
        path,
        query,
        headers,
        body,
    })
}

/// Reads the head of a request, up to and including the blank line.
///
/// 读取请求的头部，直至并包含空行。
fn read_head<R: Read>(stream: &mut R, limits: Limits) -> Result<Vec<u8>, RequestError> {
    let mut head = Vec::new();
    let mut window = [0_u8; 4];
    let mut newlines = 0_usize;
    loop {
        let mut byte = [0_u8; 1];
        match stream.read(&mut byte) {
            Ok(0) => {
                if head.is_empty() {
                    return Err(RequestError::Malformed(
                        "connection closed before a request arrived",
                    ));
                }
                return Err(RequestError::Malformed(
                    "connection closed inside the request head",
                ));
            }
            Ok(_) => head.push(byte[0]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(RequestError::Io(error)),
        }

        // Track the last four bytes so that a CRLF CRLF sequence ends the
        // head. The window is a small ring buffer over the tail of `head`.

        // 跟踪最后四个字节，使 CRLF CRLF 序列结束头部。该窗口是覆盖 `head`
        // 尾部的一个小环形缓冲。
        window.rotate_left(1);
        window[3] = byte[0];
        if window == *b"\r\n\r\n" {
            return Ok(head);
        }

        if byte[0] == b'\n' {
            newlines += 1;
            if newlines > limits.max_headers {
                return Err(RequestError::TooLarge("too many header lines"));
            }
        }
        if head.len() > limits.max_head_bytes {
            return Err(RequestError::TooLarge(
                "request head exceeds the size limit",
            ));
        }
    }
}

/// Splits a request target into its path and query components.
///
/// An absolute-form target, which a proxy would send, keeps its authority in
/// the path so that the router rejects it as an unknown path rather than
/// misreading it.
///
/// 把请求目标拆分为路径与查询两部分。
///
/// 代理会发送的绝对形式目标把其授权部分保留在路径中，使路由器把它当作未知路径
/// 拒绝，而不是误读它。
fn split_target(target: &str) -> (String, Option<String>) {
    match target.split_once('?') {
        Some((path, query)) => (path.to_string(), Some(query.to_string())),
        None => (target.to_string(), None),
    }
}
