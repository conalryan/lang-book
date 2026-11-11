//! Parsing and reporting.

use crate::models::{ParseError, Reading, Stats, TemperatureLog, Unit};

fn is_leap(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// `YYYY-MM-DDTHH:MM` with a real calendar date and a time from 00:00 to 23:59.
fn parse_timestamp(text: &str) -> Result<&str, ParseError> {
    let bad = || ParseError::BadTimestamp(text.to_string());
    let shape_ok = text.len() == 16
        && text.bytes().enumerate().all(|(i, b)| match i {
            4 | 7 => b == b'-',
            10 => b == b'T',
            13 => b == b':',
            _ => b.is_ascii_digit(),
        });
    if !shape_ok {
        return Err(bad());
    }
    let number = |range: std::ops::Range<usize>| text[range].parse::<u32>().map_err(|_| bad());
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute) = (number(11..13)?, number(14..16)?);

    let valid = year >= 1
        && (1..=12).contains(&month)
        && (1..=days_in_month(year, month)).contains(&day)
        && hour < 24
        && minute < 60;
    if valid { Ok(text) } else { Err(bad()) }
}

/// Strict decimal: Rust's `parse::<f64>` also accepts `inf`, `nan` and `infinity`.
fn parse_value(text: &str) -> Result<f64, ParseError> {
    let allowed = text
        .chars()
        .all(|c| c.is_ascii_digit() || "+-.eE".contains(c));
    match text.parse::<f64>() {
        Ok(value) if allowed && value.is_finite() => Ok(value),
        _ => Err(ParseError::BadValue(text.to_string())),
    }
}

/// Parse `<timestamp> <value> <unit>`, e.g. `2024-01-01T08:00 21.5 C`.
///
/// Each step returns a `Result`; the `?` operator returns early with the error.
pub fn parse_reading(line: &str) -> Result<Reading, ParseError> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    // `let ... else` destructures a slice pattern or bails out.
    let [timestamp, value_text, unit_text] = fields.as_slice() else {
        return Err(ParseError::FieldCount {
            found: fields.len(),
        });
    };

    let timestamp = parse_timestamp(timestamp)?;
    let value = parse_value(value_text)?;
    let unit =
        Unit::parse(unit_text).ok_or_else(|| ParseError::UnknownUnit((*unit_text).to_string()))?;
    Ok(Reading::new(timestamp, value, unit))
}

/// Build a log; bad lines become messages instead of aborting.
pub fn build_log<I>(lines: I) -> (TemperatureLog, Vec<String>)
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut log = TemperatureLog::new();
    let mut problems = Vec::new();

    for (index, line) in lines.into_iter().enumerate() {
        let line = line.as_ref();
        if line.trim().is_empty() {
            continue;
        }
        match parse_reading(line) {
            Ok(reading) => {
                log.add(reading);
            }
            Err(error) => problems.push(format!("line {}: {error}", index + 1)),
        }
    }
    (log, problems)
}

fn round2(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    match format!("{value:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
    {
        "" | "-0" => "0".to_string(),
        text => text.to_string(),
    }
}

/// Format the statistics block.
pub fn format_stats(stats: Option<&Stats>, latest: Option<&Reading>) -> Vec<String> {
    let Some(stats) = stats else {
        return vec!["count: 0".to_string()];
    };
    let (timestamp, celsius) =
        latest.map_or(("n/a", f64::NAN), |r| (r.timestamp.as_str(), r.celsius()));
    vec![
        format!("count: {}", stats.count),
        format!("min: {} C", round2(stats.min)),
        format!("max: {} C", round2(stats.max)),
        format!("mean: {} C", round2(stats.mean)),
        format!("median: {} C", round2(stats.median)),
        format!("latest: {timestamp} {} C", round2(celsius)),
    ]
}

/// Full report: problems first, then statistics.
pub fn report<I>(lines: I) -> Vec<String>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let (log, mut problems) = build_log(lines);
    problems.extend(format_stats(log.stats().as_ref(), log.latest()));
    problems
}

#[cfg(test)]
mod tests;
