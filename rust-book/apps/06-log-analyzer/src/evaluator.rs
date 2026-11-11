//! Tree-walking interpreter for filter expressions.

use std::cmp::Ordering;

use crate::models::{CompareOp, Expr, FilterSyntaxError, LEVELS, LogEntry, Value};
use crate::parser::parse;

/// Parse a strict, finite decimal, or `None`.
///
/// Rust's `parse::<f64>` also accepts `inf` and `nan`, so the characters are checked first.
pub fn to_number(text: &str) -> Option<f64> {
    let allowed = text
        .chars()
        .all(|c| c.is_ascii_digit() || "+-.eE".contains(c));
    text.parse::<f64>()
        .ok()
        .filter(|n| allowed && n.is_finite())
}

fn field_value<'a>(entry: &'a LogEntry, field: &str) -> Option<&'a str> {
    match field {
        "level" => Some(&entry.level),
        "service" => Some(&entry.service),
        "message" => Some(&entry.message),
        "timestamp" => Some(&entry.timestamp),
        _ => entry.fields.get(field).map(String::as_str),
    }
}

/// Render values the way the other ports do: `500.0` becomes `500`.
fn to_text(value: &Value) -> String {
    match value {
        Value::Text(text) => text.clone(),
        Value::Number(number) => number.to_string(),
    }
}

/// `None` when the values cannot be compared (unknown level, non-numeric text vs number).
fn ordering(actual: &str, expected: &Value, field: &str) -> Option<Ordering> {
    if field == "level" {
        let rank = |name: &str| LEVELS.iter().position(|level| *level == name);
        return Some(rank(actual)?.cmp(&rank(&to_text(expected).to_uppercase())?));
    }
    match expected {
        Value::Number(expected) => to_number(actual)?.partial_cmp(expected),
        Value::Text(expected) => Some(actual.cmp(expected.as_str())),
    }
}

fn compare(actual: Option<&str>, op: CompareOp, expected: &Value, field: &str) -> bool {
    // A missing field only satisfies "!=".
    let Some(actual) = actual else {
        return op == CompareOp::Ne;
    };
    if op == CompareOp::Contains {
        return actual.contains(&to_text(expected));
    }
    let Some(order) = ordering(actual, expected, field) else {
        return op == CompareOp::Ne;
    };

    match op {
        CompareOp::Eq => order == Ordering::Equal,
        CompareOp::Ne => order != Ordering::Equal,
        CompareOp::Lt => order == Ordering::Less,
        CompareOp::Le => order != Ordering::Greater,
        CompareOp::Gt => order == Ordering::Greater,
        CompareOp::Ge => order != Ordering::Less,
        CompareOp::Contains => false, // handled above; kept so the match stays exhaustive
    }
}

/// Pattern match on the node type and recurse. Borrows the tree: evaluating does not consume it.
pub fn evaluate(expr: &Expr, entry: &LogEntry) -> bool {
    match expr {
        Expr::And(left, right) => evaluate(left, entry) && evaluate(right, entry),
        Expr::Or(left, right) => evaluate(left, entry) || evaluate(right, entry),
        Expr::Not(operand) => !evaluate(operand, entry),
        Expr::Compare { field, op, value } => compare(field_value(entry, field), *op, value, field),
    }
}

/// Parse once, then return a closure that owns the tree (`move`) and tests entries cheaply.
pub fn compile_filter(
    source: &str,
) -> Result<impl Fn(&LogEntry) -> bool + use<>, FilterSyntaxError> {
    let tree = parse(source)?;
    Ok(move |entry: &LogEntry| evaluate(&tree, entry))
}
