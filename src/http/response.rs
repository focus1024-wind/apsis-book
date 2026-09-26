//! Building and writing outbound HTTP/1.1 responses.

use std::io::{self, Write};

/// A response status.
///
/// Only the codes the service produces are listed. The code and reason
/// phrase are kept together so that the wire format cannot disagree with the
/// meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// `200 OK`.
    Ok,
    /// `204 No Content`.
    NoContent,
    /// `400 Bad Request`.
    BadRequest,
    /// `404 Not Found`.
    NotFound,
    /// `405 Method Not Allowed`.
    MethodNotAllowed,
    /// `413 Content Too Large`.
    ContentTooLarge,
    /// `500 Internal Server Error`.
    InternalServerError,
}

impl Status {
    /// Returns the numeric status code.
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
#[derive(Debug, Clone)]
pub struct Response {
    /// Status of the response.
    pub status: Status,
    /// Media type of the body, such as `application/json`.
    pub content_type: &'static str,
    /// Body bytes.
    pub body: Vec<u8>,
    /// Value for the `Cache-Control` header, when the response may be
    /// cached.
    pub cache_control: Option<&'static str>,
    /// Extra header fields.
    pub extra_headers: Vec<(&'static str, String)>,
}

impl Response {
    /// Creates a response with the given status, media type, and body.
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
    pub fn json(body: String) -> Self {
        Self::new(
            Status::Ok,
            "application/json; charset=utf-8",
            body.into_bytes(),
        )
    }

    /// Creates a `200 OK` response carrying a UTF-8 text body.
    #[allow(dead_code)]
    pub fn text(content_type: &'static str, body: String) -> Self {
        Self::new(Status::Ok, content_type, body.into_bytes())
    }

    /// Creates a response with no body.
    pub fn empty(status: Status) -> Self {
        Self::new(status, "text/plain; charset=utf-8", Vec::new())
    }

    /// Creates a plain text error response.
    pub fn error(status: Status, message: &str) -> Self {
        Self::new(
            status,
            "text/plain; charset=utf-8",
            message.as_bytes().to_vec(),
        )
    }

    /// Sets the `Cache-Control` header.
    pub fn with_cache_control(mut self, value: &'static str) -> Self {
        self.cache_control = Some(value);
        self
    }

    /// Adds a header field.
    pub fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.extra_headers.push((name, value.into()));
        self
    }

    /// Writes the response to `stream`.
    ///
    /// `suppress_body` is set for responses to a `HEAD` request, where the
    /// headers describe the body that a `GET` would return but no body is
    /// sent.
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
    #[allow(dead_code)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();
        // Writing into a vector cannot fail.
        let _ = self.write_to(&mut output, false);
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_status_and_length() {
        let response = Response::json("{\"a\":1}".to_string());
        let text = String::from_utf8(response.to_bytes()).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(text.contains("Content-Type: application/json; charset=utf-8\r\n"));
        assert!(text.contains("Content-Length: 7\r\n"));
        assert!(text.ends_with("\r\n\r\n{\"a\":1}"));
    }

    #[test]
    fn head_response_keeps_the_length_but_drops_the_body() {
        let response = Response::json("{}".to_string());
        let mut output = Vec::new();
        response.write_to(&mut output, true).unwrap();
        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Content-Length: 2\r\n"));
        assert!(text.ends_with("\r\n\r\n"));
    }

    #[test]
    fn empty_body_has_no_content_type() {
        let response = Response::empty(Status::NoContent);
        let text = String::from_utf8(response.to_bytes()).unwrap();
        // The response carries `X-Content-Type-Options`, so the assertion is
        // on the leading header line rather than on the substring.
        assert!(!text.contains("\r\nContent-Type:"));
        assert!(text.contains("Content-Length: 0\r\n"));
    }
}
