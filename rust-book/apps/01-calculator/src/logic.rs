//! Pure behaviour: parsing, arithmetic, conversion and formatting.

use crate::models::{CalcError, Command, Operator, Unit};
use crate::tour::tour;

/// Shown for any malformed command line.
pub const USAGE: &str =
    "usage: calc <number> <op> <number> | calc convert <value> <from> <to> | calc tour";

fn error(message: impl Into<String>) -> CalcError {
    CalcError(message.into())
}

/// Parse a strict, finite decimal number.
///
/// Rust: `str::parse::<f64>` also accepts `inf`, `nan` and `infinity`, so the characters are
/// checked first.
pub fn parse_number(text: &str) -> Result<f64, CalcError> {
    let trimmed = text.trim();
    let allowed = trimmed
        .chars()
        .all(|c| c.is_ascii_digit() || "+-.eE".contains(c));
    match trimmed.parse::<f64>() {
        Ok(value) if allowed && value.is_finite() => Ok(value),
        _ => Err(error(format!("'{text}' is not a number"))),
    }
}

/// Apply `op`. `%` keeps the sign of the left operand, like Rust's own `%`.
pub fn calculate(left: f64, op: Operator, right: f64) -> Result<f64, CalcError> {
    match op {
        Operator::Add => Ok(left + right),
        Operator::Sub => Ok(left - right),
        Operator::Mul => Ok(left * right),
        Operator::Div | Operator::Rem if right == 0.0 => Err(error("division by zero")),
        Operator::Div => Ok(left / right),
        Operator::Rem => Ok(left % right),
        Operator::Pow => Ok(left.powf(right)),
    }
}

/// Convert between two units of the same dimension via the base unit.
pub fn convert(value: f64, from: Unit, to: Unit) -> Result<f64, CalcError> {
    if from.dimension() != to.dimension() {
        return Err(error(format!(
            "cannot convert {} to {}",
            from.dimension(),
            to.dimension()
        )));
    }
    Ok(to.from_base(from.to_base(value)))
}

/// Round to 6 decimals and drop trailing zeros: `0.1 + 0.2` prints as `0.3`.
pub fn format_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.into();
    }
    let text = format!("{value:.6}");
    match text.trim_end_matches('0').trim_end_matches('.') {
        "" | "-0" => "0".into(),
        trimmed => trimmed.into(),
    }
}

/// Turn raw arguments (without the program name) into a typed command.
///
/// Slice patterns destructure the argument list directly.
pub fn parse_command(args: &[String]) -> Result<Command, CalcError> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    match args.as_slice() {
        ["tour"] => Ok(Command::Tour),
        ["convert", value, from, to] => {
            let from =
                Unit::from_symbol(from).ok_or_else(|| error(format!("unknown unit '{from}'")))?;
            let to = Unit::from_symbol(to).ok_or_else(|| error(format!("unknown unit '{to}'")))?;
            Ok(Command::Convert {
                value: parse_number(value)?,
                from,
                to,
            })
        }
        ["convert", ..] => Err(error(USAGE)),
        [left, op, right] => {
            let op = Operator::from_symbol(op)
                .ok_or_else(|| error(format!("unknown operator '{op}'")))?;
            Ok(Command::Calc {
                left: parse_number(left)?,
                op,
                right: parse_number(right)?,
            })
        }
        _ => Err(error(USAGE)),
    }
}

/// Run a command and return the exact text to print.
pub fn execute(command: &Command) -> Result<String, CalcError> {
    match *command {
        Command::Calc { left, op, right } => {
            let result = calculate(left, op, right)?;
            Ok(format!(
                "{} {op} {} = {}",
                format_number(left),
                format_number(right),
                format_number(result)
            ))
        }
        Command::Convert { value, from, to } => {
            let result = convert(value, from, to)?;
            Ok(format!(
                "{} {from} = {} {to}",
                format_number(value),
                format_number(result)
            ))
        }
        Command::Tour => Ok(tour().join("\n")),
    }
}

#[cfg(test)]
mod tests;
