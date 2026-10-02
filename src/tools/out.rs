//! Helpers that shape tool results.
//!
//! A tool returns a shaped value with the columns of a table, the
//! numeric series for a plot, the records of a document, and the labelled
//! scalars that the frontend shows beside them. Keeping the shape in one
//! place means a tool describes what it computed rather than how it is
//! drawn, and the frontend needs one renderer for every tool.
//!
//! 塑造工具结果的辅助构件。
//!
//! 工具返回一个已整形的值，含表格的各列、供绘图用的数值序列、文档的各记录，以及
//! 前端在其旁显示的带标签标量。把形状集中在一处，意味着工具描述的是它算出了什么，
//! 而不是它如何被画出；前端也只需为所有工具准备一个渲染器。

use crate::i18n::Text;
use crate::json::Value;
use crate::json_object;
use crate::tools::labels;

/// A named scalar with a unit, rendered as a definition pair.
///
/// 一个带单位的具名标量，以「名—值」对的形式呈现。
#[derive(Debug, Clone)]
pub struct Scalar {
    /// Human readable name.
    /// 供人阅读的名字。
    pub label: String,
    /// Formatted value.
    /// 已格式化的取值。
    pub value: String,
    /// Unit, or an empty string.
    /// 单位，或空字符串。
    pub unit: String,
    /// True when the value should be shown in a monospaced font, which is
    /// used for identifiers and angles written in degrees and minutes.
    ///
    /// 当该值应以等宽字体显示时为真，用于标识符与以度分表示的角。
    pub monospace: bool,
}

impl Scalar {
    /// Creates a scalar with a unit.
    /// 创建一个带单位的标量。
    pub fn with_unit(label: impl Into<String>, value: impl ToString, unit: &str) -> Self {
        Self {
            label: label.into(),
            value: value.to_string(),
            unit: unit.to_string(),
            monospace: false,
        }
    }

    /// Creates a scalar without a unit.
    /// 创建一个不带单位的标量。
    pub fn plain(label: impl Into<String>, value: impl ToString) -> Self {
        Self {
            label: label.into(),
            value: value.to_string(),
            unit: String::new(),
            monospace: false,
        }
    }

    /// Marks the scalar as monospaced.
    /// 把该标量标记为等宽显示。
    pub fn monospace(mut self, monospace: bool) -> Self {
        self.monospace = monospace;
        self
    }
}

/// A table column.
///
/// 表格的一列。
#[derive(Debug, Clone)]
pub struct Column {
    /// Machine name, used as the record member name.
    /// 机器名，用作记录的成员名。
    pub key: String,
    /// Column heading.
    /// 列标题。
    pub label: String,
    /// Unit shown in the heading, or an empty string.
    /// 显示在标题中的单位，或空字符串。
    pub unit: String,
    /// True when the column holds numbers, which right aligns it and lets
    /// the frontend format it.
    ///
    /// 当该列存放数值时为真：这使它右对齐，并让前端负责格式化。
    pub numeric: bool,
}

impl Column {
    /// Creates a numeric column with a unit.
    /// 创建一个带单位的数值列。
    pub fn number(key: &str, label: &str, unit: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            unit: unit.to_string(),
            numeric: true,
        }
    }

    /// Creates a text column.
    /// 创建一个文本列。
    pub fn text(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            unit: String::new(),
            numeric: false,
        }
    }
}

/// A table of records.
///
/// 一张记录表。
#[derive(Debug, Clone, Default)]
pub struct Table {
    /// Columns, in display order.
    /// 各列，按显示顺序。
    pub columns: Vec<Column>,
    /// Rows, each holding one value per column.
    /// 各行，每行对每一列持有一个值。
    pub rows: Vec<Vec<Value>>,
}

impl Table {
    /// Creates a table with the given columns.
    /// 以给定的列创建一张表。
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
        }
    }

    /// Appends a row.
    /// 追加一行。
    pub fn push(&mut self, row: Vec<Value>) {
        self.rows.push(row);
    }

    /// Returns the column at a position, if there is one.
    ///
    /// A row is written by the handler, which may add or leave out a cell, so
    /// a position is not guaranteed to name a column.
    ///
    /// 返回某个位置上的列，若有。
    ///
    /// 行由处理函数书写，它可能多写或少写一个单元格，因此一个位置并不保证对应
    /// 某一列。
    pub fn column(&self, index: usize) -> Option<&Column> {
        self.columns.get(index)
    }

    /// Returns `true` when the table has no rows.
    /// 当表中没有行时返回 `true`。
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// A named numeric series, rendered as a chart or a sparkline.
#[derive(Debug, Clone)]
pub struct Series {
    /// Series name.
    pub name: String,
    /// Unit of the values.
    pub unit: String,
    /// Values in order.
    pub values: Vec<f64>,
}

impl Series {
    /// Creates a series.
    pub fn new(name: impl Into<String>, unit: &str, values: Vec<f64>) -> Self {
        Self {
            name: name.into(),
            unit: unit.to_string(),
            values,
        }
    }
}

/// Formats a number for display with a fixed number of decimals.
pub fn fmt(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

/// Converts degrees to radians.
pub fn radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Converts radians to degrees.
pub fn degrees(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

/// Builds the response of a tool that produces a table.
///
/// The response always carries a `kind` member so that the frontend can pick
/// a renderer, plus the optional `scalars`, `series`, `records`, and `notes`
/// members that the tool supplied.
pub fn table_response(
    kind: &str,
    title: &str,
    scalars: Vec<Scalar>,
    table: &Table,
    series: Vec<Series>,
    notes: Vec<Text>,
) -> Value {
    let columns: Vec<Value> = table
        .columns
        .iter()
        .map(|column| {
            json_object! {
                "key" => column.key.clone(),
                "label" => labels::label(&column.label).json(),
                "unit" => column.unit.clone(),
                "unit_html" => unit_html(&column.unit),
                "numeric" => column.numeric,
            }
        })
        .collect();
    // A text cell may hold a fixed phrase rather than data: the meaning of a
    // field, for instance. Those are looked up so that a reader of either
    // language sees prose in that language. A cell holding data was written
    // by the reader rather than by this table, so the lookup gives it back
    // unchanged in both languages, which is what it should show.
    let rows: Vec<Value> = table
        .rows
        .iter()
        .map(|row| {
            Value::Array(
                row.iter()
                    .enumerate()
                    .map(|(index, cell)| match cell {
                        Value::String(text)
                            if !table.column(index).is_some_and(|column| column.numeric) =>
                        {
                            let mut encoded = labels::label(text).json();
                            // A unit is written with a power in it, as in
                            // `rev/day^2`, and a unit is the one kind of text a
                            // table shows that is notation rather than prose.
                            // Only notation is rendered, so a sentence is never
                            // set as a formula and read as a row of variables.
                            if is_notation(text) {
                                if let Value::Object(fields) = &mut encoded {
                                    fields.push((
                                        "html".to_string(),
                                        Value::from(crate::docs::mathml::render(text, false)),
                                    ));
                                }
                            }
                            encoded
                        }
                        other => other.clone(),
                    })
                    .collect(),
            )
        })
        .collect();
    let scalar_values: Vec<Value> = scalars.iter().map(scalar_json).collect();
    let series_values: Vec<Value> = series.iter().map(series_json).collect();

    json_object! {
        "kind" => kind,
        "title" => labels::label(title).json(),
        "scalars" => scalar_values,
        "table" => json_object! { "columns" => columns, "rows" => rows },
        "series" => series_values,
        // A note is written in Markdown, so it is sent as the source and as
        // HTML in both languages; a formula in a note is set the same way as
        // one in a document.
        "notes" => notes.iter().map(Text::json).collect::<Vec<Value>>(),
        "notes_html" => notes.iter().map(note_html).collect::<Vec<Value>>(),
    }
}

/// Renders one note to HTML in both languages.
fn note_html(note: &Text) -> Value {
    json_object! {
        "zh" => crate::docs::markdown::render(note.zh),
        "en" => crate::docs::markdown::render(note.en),
    }
}

/// Reports whether a piece of text is notation that should be set.
///
/// Only a unit qualifies. A unit is a single token carrying a power, such as
/// `rev/day^2`, and a power is what a reader needs to see raised.
///
/// Three things are deliberately excluded. A sentence such as `Mean motion`
/// is prose, and a formula sets its letters as variables, so a sentence read
/// that way stops being a sentence. An identifier such as `mean_motion` is a
/// field name, and it is already written the way the field is spelled; setting
/// it as a formula would split it into letters and turn the underscore into a
/// subscript. And a unit written with a space is prose with a caret in it,
/// which is not a unit.
fn is_notation(text: &str) -> bool {
    text.contains('^') && !text.contains(' ')
}

/// Renders a unit as HTML, so that a power reads as a power.
///
/// A unit is notation rather than prose: `rev/day^2` is a name, and the only
/// mark-up it ever needs is the exponent. It is rendered by the same
/// mathematics renderer as a formula so that the two agree on how a power
/// looks, and so that a unit written with a subscript is set correctly too.
fn unit_html(unit: &str) -> Value {
    if unit.is_empty() {
        return Value::string(String::new());
    }
    Value::from(crate::docs::mathml::render(unit, false))
}

/// Returns a note written as a literal in the label table.
///
/// The note text is both the lookup key and the English form, so a note
/// that is a fixed sentence is written once.
pub fn note(english: &str) -> Text {
    labels::label(english)
}

/// Pairs an English note with its Chinese form.
///
/// Used for a note that carries a value, such as a gravity model name, where
/// the two languages differ in more than a table lookup can express.
pub fn note_pair(english: impl Into<String>, chinese: impl Into<String>) -> Text {
    Text::new(
        Box::leak(chinese.into().into_boxed_str()),
        Box::leak(english.into().into_boxed_str()),
    )
}

/// Encodes a scalar.
fn scalar_json(scalar: &Scalar) -> Value {
    json_object! {
        "label" => labels::label(&scalar.label).json(),
        "value" => scalar.value.clone(),
        "unit" => scalar.unit.clone(),
        "unit_html" => unit_html(&scalar.unit),
        "monospace" => scalar.monospace,
    }
}

/// Encodes a series.
fn series_json(series: &Series) -> Value {
    let values: Vec<Value> = series
        .values
        .iter()
        .map(|value| Value::from(*value))
        .collect();
    json_object! {
        "name" => series.name.clone(),
        "unit" => series.unit.clone(),
        "values" => values,
    }
}
