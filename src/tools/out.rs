//! Helpers that shape tool results.
//!
//! A tool returns a shaped value with the columns of a table, the
//! numeric series for a plot, the records of a document, and the labelled
//! scalars that the frontend shows beside them. Keeping the shape in one
//! place means a tool describes what it computed rather than how it is
//! drawn, and the frontend needs one renderer for every tool.

use crate::i18n::Text;
use crate::json::Value;
use crate::json_object;
use crate::tools::labels;

/// A named scalar with a unit, rendered as a definition pair.
#[derive(Debug, Clone)]
pub struct Scalar {
    /// Human readable name.
    pub label: String,
    /// Formatted value.
    pub value: String,
    /// Unit, or an empty string.
    pub unit: String,
    /// True when the value should be shown in a monospaced font, which is
    /// used for identifiers and angles written in degrees and minutes.
    pub monospace: bool,
}

impl Scalar {
    /// Creates a scalar with a unit.
    pub fn with_unit(label: impl Into<String>, value: impl ToString, unit: &str) -> Self {
        Self {
            label: label.into(),
            value: value.to_string(),
            unit: unit.to_string(),
            monospace: false,
        }
    }

    /// Creates a scalar without a unit.
    pub fn plain(label: impl Into<String>, value: impl ToString) -> Self {
        Self {
            label: label.into(),
            value: value.to_string(),
            unit: String::new(),
            monospace: false,
        }
    }

    /// Marks the scalar as monospaced.
    pub fn monospace(mut self, monospace: bool) -> Self {
        self.monospace = monospace;
        self
    }
}

/// A table column.
#[derive(Debug, Clone)]
pub struct Column {
    /// Machine name, used as the record member name.
    pub key: String,
    /// Column heading.
    pub label: String,
    /// Unit shown in the heading, or an empty string.
    pub unit: String,
    /// True when the column holds numbers, which right aligns it and lets
    /// the frontend format it.
    pub numeric: bool,
}

impl Column {
    /// Creates a numeric column with a unit.
    pub fn number(key: &str, label: &str, unit: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            unit: unit.to_string(),
            numeric: true,
        }
    }

    /// Creates a text column.
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
#[derive(Debug, Clone, Default)]
pub struct Table {
    /// Columns, in display order.
    pub columns: Vec<Column>,
    /// Rows, each holding one value per column.
    pub rows: Vec<Vec<Value>>,
}

impl Table {
    /// Creates a table with the given columns.
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
        }
    }

    /// Appends a row.
    pub fn push(&mut self, row: Vec<Value>) {
        self.rows.push(row);
    }

    /// Returns the column at a position, if there is one.
    ///
    /// A row is written by the handler, which may add or leave out a cell, so
    /// a position is not guaranteed to name a column.
    pub fn column(&self, index: usize) -> Option<&Column> {
        self.columns.get(index)
    }

    /// Returns `true` when the table has no rows.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The first cell of the first row of a shaped response.
    fn first_cell(value: &Value) -> Value {
        let rows = value
            .get("table")
            .and_then(|table| table.get("rows"))
            .expect("the response carries a table")
            .as_array();
        rows.first()
            .and_then(|row| row.as_array().first())
            .expect("the table carries a row")
            .clone()
    }

    #[test]
    fn a_text_cell_is_shown_in_either_language() {
        // The meaning of a field is a fixed phrase, so it is looked up rather
        // than written twice.
        let mut table = Table::new(vec![Column::text("meaning", "Meaning")]);
        table.push(vec![Value::from("Mean motion")]);
        let value = table_response(
            "kind",
            "Title",
            Vec::new(),
            &table,
            Vec::new(),
            vec![labels::label("Mean motion")],
        );
        let cell = first_cell(&value);
        assert_eq!(cell.get("zh").and_then(Value::as_str), Some("平均运动"));
        assert_eq!(cell.get("en").and_then(Value::as_str), Some("Mean motion"));
    }

    #[test]
    fn an_identifier_is_not_set_as_a_formula() {
        // `mean_motion` is a field name. Set as a formula it would come out as
        // separate letters with a subscript, which is not the field name.
        let mut table = Table::new(vec![Column::text("field", "Field")]);
        table.push(vec![Value::from("mean_motion")]);
        let value = table_response("kind", "Title", Vec::new(), &table, Vec::new(), Vec::new());
        let cell = first_cell(&value);
        assert!(
            cell.get("html").is_none(),
            "an identifier should not be rendered: {cell:?}"
        );
    }

    #[test]
    fn a_unit_with_a_power_is_rendered() {
        // `rev/day^2` is a unit, and the power is what a reader needs raised.
        let mut table = Table::new(vec![Column::text("unit", "Unit")]);
        table.push(vec![Value::from("rev/day^2")]);
        let value = table_response("kind", "Title", Vec::new(), &table, Vec::new(), Vec::new());
        let cell = first_cell(&value);
        let html = cell.get("html").and_then(Value::as_str).unwrap_or_default();
        assert!(html.contains("<msup>"), "the power was not raised: {html}");
    }

    #[test]
    fn table_response_carries_the_shape() {
        let mut table = Table::new(vec![Column::number("a", "A", "km")]);
        table.push(vec![Value::from(1.0)]);
        let value = table_response(
            "table",
            "T",
            vec![Scalar::plain("x", 1)],
            &table,
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(value.get("kind").unwrap().as_str(), Some("table"));
        assert_eq!(
            value
                .get("table")
                .unwrap()
                .get("rows")
                .unwrap()
                .as_array()
                .len(),
            1
        );
        assert_eq!(value.get("scalars").unwrap().as_array().len(), 1);
    }

    #[test]
    fn conversions_round_trip() {
        assert!((degrees(radians(30.0)) - 30.0).abs() < 1e-12);
    }
}
