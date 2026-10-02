//! Building and writing outbound HTTP/1.1 responses.
//!
//! 出站 HTTP/1.1 响应的构造与写出。

use std::io::{self, Write};

/// A response status.
///
/// Only the codes the service produces are listed. The code and reason
/// phrase are kept together so that the wire format cannot disagree with the
/// meaning.
///
/// 一个响应状态。
///
/// 只列出服务会产生的状态码。状态码与原因短语放在一起，使线路格式不可能与其含义
/// 不一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// `200 OK`.
    /// `200 OK`。
    Ok,
    /// `204 No Content`.
    /// `204 No Content`。
    NoContent,
    /// `400 Bad Request`.
    /// `400 Bad Request`。
    BadRequest,
    /// `404 Not Found`.
    /// `404 Not Found`。
    NotFound,
    /// `405 Method Not Allowed`.
    /// `405 Method Not Allowed`。
    MethodNotAllowed,
    /// `413 Content Too Large`.
    /// `413 Content Too Large`。
    ContentTooLarge,
    /// `500 Internal Server Error`.
    /// `500 Internal Server Error`。
    InternalServerError,
}

impl Status {
    /// Returns the numeric status code.
    /// 返回数字状态码。
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::NoContent => 204,
            Self::BadRequest => 400,
            Self::NotFound => 404,
            Self::MethodNotAllowed => 405,
            Self::ContentTooLarge => 413,
            Self::InternalServerError => 500,
        }
    }

    /// Returns the reason phrase sent with the status code.
    /// 返回随状态码一同发送的原因短语。
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::NoContent => "No Content",
            Self::BadRequest => "Bad Request",
            Self::NotFound => "Not Found",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::ContentTooLarge => "Content Too Large",
            Self::InternalServerError => "Internal Server Error",
        }
    }
}

/// A response ready to be written.
///
/// The content type names the media type of the body; a body of zero length
/// is sent without a content type. Only the headers the service needs are
/// modelled, which keeps the writer a single pass with no conditional logic.
///
/// 一个可以写出的响应。
///
/// 内容类型给出请求体的媒体类型；长度为零的请求体不带内容类型发送。只为服务所需
/// 的首部建模，这使写出过程是一趟线性过程而不含条件分支。
#[derive(Debug, Clone)]
pub struct Response {
    /// Status of the response.
    /// 响应的状态。
    pub status: Status,
    /// Media type of the body, such as `application/json`.
    /// 请求体的媒体类型，例如 `application/json`。
    pub content_type: &'static str,
    /// Body bytes.
    /// 请求体字节。
    pub body: Vec<u8>,
    /// Value for the `Cache-Control` header, when the response may be
    /// cached.
    ///
    /// `Cache-Control` 首部的取值，当该响应可被缓存时。
    pub cache_control: Option<&'static str>,
    /// Extra header fields.
    /// 额外的首部字段。
    pub extra_headers: Vec<(&'static str, String)>,
}

impl Response {
    /// Creates a response with the given status, media type, and body.
    /// 以给定的状态、媒体类型与请求体创建一个响应。
    pub fn new(status: Status, content_type: &'static str, body: Vec<u8>) -> Self {
        Self {
            status,
            content_type,
            body,
            cache_control: None,
            extra_headers: Vec::new(),
        }
    }

    /// Creates a `200 OK` response carrying JSON text.
    /// 创建一个携带 JSON 文本的 `200 OK` 响应。
    pub fn json(body: String) -> Self {
        Self::new(
            Status::Ok,
            "application/json; charset=utf-8",
            body.into_bytes(),
        )
    }

    /// Creates a `200 OK` response carrying a UTF-8 text body.
    /// 创建一个携带 UTF-8 文本体的 `200 OK` 响应。
    #[allow(dead_code)]
    pub fn text(content_type: &'static str, body: String) -> Self {
        Self::new(Status::Ok, content_type, body.into_bytes())
    }

    /// Creates a response with no body.
    /// 创建一个没有请求体的响应。
    pub fn empty(status: Status) -> Self {
        Self::new(status, "text/plain; charset=utf-8", Vec::new())
    }

    /// Creates a plain text error response.
    /// 创建一个纯文本错误响应。
    pub fn error(status: Status, message: &str) -> Self {
        Self::new(
            status,
            "text/plain; charset=utf-8",
            message.as_bytes().to_vec(),
        )
    }

    /// Sets the `Cache-Control` header.
    /// 设置 `Cache-Control` 首部。
    pub fn with_cache_control(mut self, value: &'static str) -> Self {
        self.cache_control = Some(value);
        self
    }

    /// Adds a header field.
    /// 添加一个首部字段。
    pub fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.extra_headers.push((name, value.into()));
        self
    }

    /// Writes the response to `stream`.
    ///
    /// `suppress_body` is set for responses to a `HEAD` request, where the
    /// headers describe the body that a `GET` would return but no body is
    /// sent.
    ///
    /// 把响应写到 `stream`。
    ///
    /// 对 `HEAD` 请求的响应会置上 `suppress_body`：此时首部描述 `GET` 会返回的
    /// 请求体，但不发送请求体本身。
    pub fn write_to<W: Write>(&self, stream: &mut W, suppress_body: bool) -> io::Result<()> {
        let mut head = String::new();
        head.push_str("HTTP/1.1 ");
        head.push_str(&self.status.code().to_string());
        head.push(' ');
        head.push_str(self.status.reason());
        head.push_str("\r\n");
        if !self.body.is_empty() {
            head.push_str("Content-Type: ");
            head.push_str(self.content_type);
            head.push_str("\r\n");
        }
        head.push_str("Content-Length: ");
        head.push_str(&self.body.len().to_string());
        head.push_str("\r\n");
        if let Some(value) = self.cache_control {
            head.push_str("Cache-Control: ");
            head.push_str(value);
            head.push_str("\r\n");
        }
        for (name, value) in &self.extra_headers {
            head.push_str(name);
            head.push_str(": ");
            head.push_str(value);
            head.push_str("\r\n");
        }
        head.push_str("X-Content-Type-Options: nosniff\r\n");
        head.push_str("Connection: close\r\n\r\n");

        stream.write_all(head.as_bytes())?;
        if !suppress_body && !self.body.is_empty() {
            stream.write_all(&self.body)?;
        }
        stream.flush()
    }

    /// Returns the response as bytes, primarily for tests.
    /// 把响应返回为字节，主要用于测试。
    #[allow(dead_code)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();
        // Writing into a vector cannot fail.
        // 向一个向量写入不会失败。
        let _ = self.write_to(&mut output, false);
        output
    }
}
