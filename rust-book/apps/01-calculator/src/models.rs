//! Types: operators, units and the parsed command.

use std::fmt;

/// Binary operators. A closed set, so an `enum` (TypeScript used a union of string literals).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
}

impl Operator {
    /// Look an operator up by its symbol, e.g. `"**"`.
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "+" => Some(Self::Add),
            "-" => Some(Self::Sub),
            "*" => Some(Self::Mul),
            "/" => Some(Self::Div),
            "%" => Some(Self::Rem),
            "**" => Some(Self::Pow),
            _ => None,
        }
    }

    /// The symbol as typed on the command line.
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Rem => "%",
            Self::Pow => "**",
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

/// What a unit measures; only units of the same dimension convert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    Length,
    Mass,
    Temperature,
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Length => "length",
            Self::Mass => "mass",
            Self::Temperature => "temperature",
        })
    }
}

/// Supported units. Methods live in an `impl` block, separate from the data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Km,
    Mi,
    Kg,
    Lb,
    C,
    F,
}

impl Unit {
    /// Look a unit up by its lower-case symbol.
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "km" => Some(Self::Km),
            "mi" => Some(Self::Mi),
            "kg" => Some(Self::Kg),
            "lb" => Some(Self::Lb),
            "c" => Some(Self::C),
            "f" => Some(Self::F),
            _ => None,
        }
    }

    /// The symbol as typed on the command line.
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Km => "km",
            Self::Mi => "mi",
            Self::Kg => "kg",
            Self::Lb => "lb",
            Self::C => "c",
            Self::F => "f",
        }
    }

    /// What this unit measures.
    pub fn dimension(self) -> Dimension {
        match self {
            Self::Km | Self::Mi => Dimension::Length,
            Self::Kg | Self::Lb => Dimension::Mass,
            Self::C | Self::F => Dimension::Temperature,
        }
    }

    /// Convert into the base unit (km, kg, celsius).
    pub fn to_base(self, value: f64) -> f64 {
        match self {
            Self::Km | Self::Kg | Self::C => value,
            Self::Mi => value * 1.609_344,
            Self::Lb => value * 0.453_592_37,
            Self::F => ((value - 32.0) * 5.0) / 9.0,
        }
    }

    /// Convert from the base unit (km, kg, celsius).
    pub fn from_base(self, value: f64) -> f64 {
        match self {
            Self::Km | Self::Kg | Self::C => value,
            Self::Mi => value / 1.609_344,
            Self::Lb => value / 0.453_592_37,
            Self::F => (value * 9.0) / 5.0 + 32.0,
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

/// A parsed command line. Each variant carries its own data, which is what makes Rust enums
/// more powerful than TypeScript or Python enums.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    /// `calc 3 + 4`
    Calc { left: f64, op: Operator, right: f64 },
    /// `calc convert 100 c f`
    Convert { value: f64, from: Unit, to: Unit },
    /// `calc tour`
    Tour,
}

/// Bad user input. The message is printed verbatim after `error: `.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalcError(pub String);

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CalcError {}
