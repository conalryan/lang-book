//! Log parsing, analysis and reporting.

use std::collections::HashMap;
use std::hash::Hash;

use crate::evaluator::{compile_filter, to_number};
use crate::models::{FilterSyntaxError, LEVELS, LogEntry, Summary};

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Find every `key=value` pair: a run of word characters, `=`, then a non-space run.
fn extract_fields(message: &str) -> HashMap<String, String> {
    let chars: Vec<char> = message.chars().collect();
    let mut fields = HashMap::new();
    let mut i = 0;

    while i < chars.len() {
        if !is_word(chars[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && is_word(chars[i]) {
            i += 1;
        }
        if chars.get(i) == Some(&'=') && chars.get(i + 1).is_some_and(|c| !c.is_whitespace()) {
            let value_start = i + 1;
            let mut end = value_start;
            while end < chars.len() && !chars[end].is_whitespace() {
                end += 1;
            }
            fields.insert(
                chars[start..i].iter().collect(),
                chars[value_start..end].iter().collect(),
            );
            i = end;
        }
    }
    fields
}

/// Parse `<timestamp> <LEVEL> <service> <message>`, or return `None` if the line does not match.
///
/// `split_once` peels one space-separated field at a time; the leftovers are borrowed slices.
pub fn parse_line(line: &str) -> Option<LogEntry> {
    let no_space = |s: &str| !s.is_empty() && !s.chars().any(char::is_whitespace);

    let (timestamp, rest) = line.split_once(' ')?;
    let (level, rest) = rest.split_once(' ')?;
    let (service, message) = rest.split_once(' ')?;

    if !no_space(timestamp) || !LEVELS.contains(&level) || !no_space(service) {
        return None;
    }
    Some(LogEntry {
        raw: line.to_string(),
        timestamp: timestamp.to_string(),
        level: level.to_string(),
        service: service.to_string(),
        message: message.to_string(),
        fields: extract_fields(message),
    })
}

/// Lazy generic filter: nothing runs until the consumer pulls values.
pub fn filter_lazily<T>(
    items: impl IntoIterator<Item = T>,
    predicate: impl Fn(&T) -> bool,
) -> impl Iterator<Item = T> {
    items.into_iter().filter(move |item| predicate(item))
}

/// Return `(entries, skipped_count)`. Blank lines are ignored, not skipped.
pub fn parse_entries<I>(lines: I) -> (Vec<LogEntry>, usize)
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut entries = Vec::new();
    let mut skipped = 0;
    for line in lines {
        let line = line.as_ref();
        if line.trim().is_empty() {
            continue;
        }
        match parse_line(line) {
            Some(entry) => entries.push(entry),
            None => skipped += 1,
        }
    }
    (entries, skipped)
}

/// `filter: None` matches everything. Fails on a bad filter, before reading any entries.
pub fn analyze<I>(lines: I, filter: Option<&str>) -> Result<Summary, FilterSyntaxError>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let predicate = filter.map(compile_filter).transpose()?;
    let (entries, skipped) = parse_entries(lines);
    let total = entries.len();
    let matched =
        filter_lazily(entries, |entry| predicate.as_ref().is_none_or(|p| p(entry))).collect();
    Ok(Summary {
        total,
        skipped,
        matched,
    })
}

/// Count items by key, in first-seen order.
pub fn count_by<'a, T: 'a, K: Eq + Hash + Clone>(
    items: impl IntoIterator<Item = &'a T>,
    key: impl Fn(&T) -> K,
) -> Vec<(K, usize)> {
    let mut index: HashMap<K, usize> = HashMap::new();
    let mut counts: Vec<(K, usize)> = Vec::new();
    for item in items {
        let k = key(item);
        if let Some(&position) = index.get(&k) {
            counts[position].1 += 1;
        } else {
            index.insert(k.clone(), counts.len());
            counts.push((k, 1));
        }
    }
    counts
}

/// Mean of the numeric `ms` fields, or `None` if there are none.
pub fn average_ms(entries: &[LogEntry]) -> Option<f64> {
    let values: Vec<f64> = entries
        .iter()
        .filter_map(|e| to_number(e.fields.get("ms")?))
        .collect();
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn trim_decimal(value: f64) -> String {
    format!("{value:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

/// Render the summary as lines.
pub fn format_report(summary: &Summary, print_matches: bool) -> Vec<String> {
    let matched = &summary.matched;
    let mut lines = Vec::new();

    if print_matches {
        lines.extend(matched.iter().map(|entry| entry.raw.clone()));
    }
    lines.push(format!("matched: {} of {}", matched.len(), summary.total));

    if !matched.is_empty() {
        let by_level = count_by(matched, |e| e.level.clone());
        let levels: Vec<String> = LEVELS
            .iter()
            .rev()
            .filter_map(|level| {
                by_level
                    .iter()
                    .find(|(name, _)| name == level)
                    .map(|(_, count)| format!("{level}={count}"))
            })
            .collect();
        lines.push(format!("levels: {}", levels.join(" ")));

        let mut by_service = count_by(matched, |e| e.service.clone());
        by_service.sort_by(|(a, x), (b, y)| {
            y.cmp(x)
                .then_with(|| a.to_lowercase().cmp(&b.to_lowercase()))
        });
        let services: Vec<String> = by_service
            .iter()
            .map(|(name, count)| format!("{name}={count}"))
            .collect();
        lines.push(format!("services: {}", services.join(" ")));

        if let Some(avg) = average_ms(matched) {
            lines.push(format!("avg ms: {}", trim_decimal(avg)));
        }
    }

    if summary.skipped > 0 {
        lines.push(format!("skipped: {}", summary.skipped));
    }
    lines
}

#[cfg(test)]
mod tests;
