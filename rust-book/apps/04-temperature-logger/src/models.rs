//! Data model: units, parse errors, readings and the log.

use std::fmt;

/// A plain enum: a closed set of named values (here with no payload).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Celsius,
    Fahrenheit,
    Kelvin,
}

impl Unit {
    /// Parse `C`, `F` or `K`, case-insensitively.
    pub fn parse(text: &str) -> Option<Self> {
        match text.to_uppercase().as_str() {
            "C" => Some(Self::Celsius),
            "F" => Some(Self::Fahrenheit),
            "K" => Some(Self::Kelvin),
            _ => None,
        }
    }
}

/// Every way a line can be rejected. Variants carry different data: a *tagged union*, which
/// TypeScript and Python can only emulate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    FieldCount { found: usize },
    BadTimestamp(String),
    BadValue(String),
    UnknownUnit(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldCount { found } => write!(f, "expected 3 fields, found {found}"),
            Self::BadTimestamp(text) => write!(f, "invalid timestamp '{text}'"),
            Self::BadValue(text) => write!(f, "invalid temperature '{text}'"),
            Self::UnknownUnit(text) => write!(f, "unknown unit '{text}'"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Summary statistics in Celsius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
}

/// A measurement. `value` is stored as entered; [`Reading::celsius`] normalises it.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub timestamp: String,
    pub value: f64,
    pub unit: Unit,
}

impl Reading {
    /// Build a reading. An associated function (no `self`): Rust's version of a constructor.
    pub fn new(timestamp: impl Into<String>, value: f64, unit: Unit) -> Self {
        Self {
            timestamp: timestamp.into(),
            value,
            unit,
        }
    }

    /// The value converted to Celsius.
    pub fn celsius(&self) -> f64 {
        match self.unit {
            Unit::Celsius => self.value,
            Unit::Fahrenheit => ((self.value - 32.0) * 5.0) / 9.0,
            Unit::Kelvin => self.value - 273.15,
        }
    }
}

/// Append-only collection of readings with derived statistics.
///
/// The field is private (no `pub`), so only this module can touch it: privacy is the default.
#[derive(Debug, Default)]
pub struct TemperatureLog {
    readings: Vec<Reading>,
}

impl TemperatureLog {
    /// An empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append and return `&mut self` so calls can be chained.
    pub fn add(&mut self, reading: Reading) -> &mut Self {
        self.readings.push(reading);
        self
    }

    /// Number of readings.
    pub fn len(&self) -> usize {
        self.readings.len()
    }

    /// True when there are no readings.
    pub fn is_empty(&self) -> bool {
        self.readings.is_empty()
    }

    /// Newest by timestamp (ties go to the later insertion); `None` when empty.
    ///
    /// Returns a reference into the log, not a copy.
    pub fn latest(&self) -> Option<&Reading> {
        self.readings
            .iter()
            .max_by(|a, b| a.timestamp.cmp(&b.timestamp))
    }

    /// `None` when empty: the type forces callers to handle "no data".
    pub fn stats(&self) -> Option<Stats> {
        if self.readings.is_empty() {
            return None;
        }
        let mut values: Vec<f64> = self.readings.iter().map(Reading::celsius).collect();
        values.sort_by(f64::total_cmp);
        let count = values.len();
        let mid = count / 2;
        let median = if count % 2 == 1 {
            values[mid]
        } else {
            f64::midpoint(values[mid - 1], values[mid])
        };
        Some(Stats {
            count,
            min: values[0],
            max: values[count - 1],
            mean: values.iter().sum::<f64>() / count as f64,
            median,
        })
    }
}
