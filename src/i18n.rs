//! Bilingual text.
//!
//! The user interface is offered in Chinese and English. The language is a
//! choice the reader makes in the browser, so the backend cannot select it
//! when it builds a payload. Instead, every string that a reader sees is
//! carried in both languages and the frontend picks one.
//!
//! A [`Text`] holds a Chinese and an English string. The service encodes it
//! as `{"zh": "...", "en": "..."}`. Payloads that carry data rather than
//! prose, such as a table of numbers or a signature, are not translated;
//! units are written with the symbols the standards use, which are the same
//! in both languages.
//!
//! The Chinese string is written first everywhere so that the order matches
//! the default language.

use crate::json::Value;
use crate::json_object;

/// A string in Chinese and English.
///
/// The two fields hold static text, so a `Text` can be declared in a
/// constant table such as the tool registry. Use [`Text::new`] to build one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Text {
    /// Simplified Chinese text.
    pub zh: &'static str,
    /// English text.
    pub en: &'static str,
}

impl Text {
    /// Creates a bilingual string.
    pub const fn new(zh: &'static str, en: &'static str) -> Self {
        Self { zh, en }
    }

    /// Encodes the pair as a JSON object with a `zh` and an `en` member.
    pub fn json(&self) -> Value {
        json_object! { "zh" => self.zh, "en" => self.en }
    }
}
