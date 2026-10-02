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
//!
//! 双语文本。
//!
//! 用户界面提供中文与英文。语言是读者在浏览器中作出的选择，因此后端在构造载荷时
//! 无法替它决定。于是读者看到的每一条字符串都以两种语言携带，由前端选取其一。
//!
//! [`Text`] 持有一个中文串与一个英文串。服务把它编码为
//! `{"zh": "...", "en": "..."}`。承载数据而非散文的载荷——例如数值表格或签名
//! ——不做翻译；单位按标准所用的符号书写，两种语言中相同。
//!
//! 各处都把中文串写在前面，以使顺序与默认语言一致。

use crate::json::Value;
use crate::json_object;

/// A string in Chinese and English.
///
/// The two fields hold static text, so a `Text` can be declared in a
/// constant table such as the tool registry. Use [`Text::new`] to build one.
///
/// 一个兼有中文与英文的字符串。
///
/// 两个字段持有静态文本，因此 `Text` 可以声明在常量表中，例如工具注册表。
/// 用 [`Text::new`] 构造一个。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Text {
    /// Simplified Chinese text.
    ///
    /// 简体中文文本。
    pub zh: &'static str,
    /// English text.
    ///
    /// 英文文本。
    pub en: &'static str,
}

impl Text {
    /// Creates a bilingual string.
    ///
    /// 创建一个双语字符串。
    pub const fn new(zh: &'static str, en: &'static str) -> Self {
        Self { zh, en }
    }

    /// Encodes the pair as a JSON object with a `zh` and an `en` member.
    ///
    /// 把该对编码为带 `zh` 与 `en` 成员的 JSON 对象。
    pub fn json(&self) -> Value {
        json_object! { "zh" => self.zh, "en" => self.en }
    }
}
