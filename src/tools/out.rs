//! Helpers that shape tool results.
//!
//! A tool returns a shaped value with the columns of a table, the
//! numeric series for a plot, the records of a document, and the labelled
//! scalars that the frontend shows beside them. Keeping the shape in one
//! place means a tool describes what it computed rather than how it is
//! drawn, and the frontend needs one renderer for every tool.

use crate::json::Value;
use crate::json_object;

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
    notes: Vec<String>,
) -> Value {
    let columns: Vec<Value> = table
        .columns
        .iter()
        .map(|column| {
            json_object! {
                "key" => column.key.clone(),
                "label" => column.label.clone(),
                "unit" => column.unit.clone(),
                "numeric" => column.numeric,
            }
        })
        .collect();
    let rows: Vec<Value> = table
        .rows
        .iter()
        .map(|row| Value::Array(row.clone()))
        .collect();
    let scalar_values: Vec<Value> = scalars.iter().map(scalar_json).collect();
    let series_values: Vec<Value> = series.iter().map(series_json).collect();

    json_object! {
        "kind" => kind,
        "title" => title,
        "scalars" => scalar_values,
        "table" => json_object! { "columns" => columns, "rows" => rows },
        "series" => series_values,
        "notes" => notes.into_iter().map(Value::from).collect::<Vec<Value>>(),
    }
}

/// Encodes a scalar.
fn scalar_json(scalar: &Scalar) -> Value {
    json_object! {
        "label" => scalar.label.clone(),
        "value" => scalar.value.clone(),
        "unit" => scalar.unit.clone(),
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
