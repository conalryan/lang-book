//! Types: log entries, tokens, the filter syntax tree and errors.

use std::collections::HashMap;
use std::fmt;

/// Ordered by severity; the index doubles as the rank used by `level >= WARN`.
pub const LEVELS: [&str; 4] = ["DEBUG", "INFO", "WARN", "ERROR"];

/// One parsed log line. It owns its strings, so it can outlive the input buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub raw: String,
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
    /// `key=value` pairs found in the message, e.g. `status=500`.
    pub fields: HashMap<String, String>,
}

/// Comparison operators of the filter language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Contains,
}

impl CompareOp {
    /// The operator as written in a filter.
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Contains => "~",
        }
    }
}

/// Boolean keywords (case-insensitive in the source).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    And,
    Or,
    Not,
}

/// What kind of token this is, with its payload.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Str(String),
    Number(String),
    Op(CompareOp),
    Keyword(Keyword),
    LParen,
    RParen,
    End,
}

/// A lexical token. `pos` is the character offset in the filter source.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: usize,
}

impl Token {
    /// The source text of the token, for error messages.
    pub fn text(&self) -> String {
        match &self.kind {
            TokenKind::Ident(s) | TokenKind::Str(s) | TokenKind::Number(s) => s.clone(),
            TokenKind::Op(op) => op.symbol().to_string(),
            TokenKind::Keyword(Keyword::And) => "and".to_string(),
            TokenKind::Keyword(Keyword::Or) => "or".to_string(),
            TokenKind::Keyword(Keyword::Not) => "not".to_string(),
            TokenKind::LParen => "(".to_string(),
            TokenKind::RParen => ")".to_string(),
            TokenKind::End => String::new(),
        }
    }
}

/// The right-hand side of a comparison.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
}

/// Filter syntax tree. A recursive enum: the `Box` gives each child a fixed-size pointer so the
/// type has a finite size. This ownership detail is the main Rust-specific part of an AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Compare {
        field: String,
        op: CompareOp,
        value: Value,
    },
}

/// A malformed filter expression; `position` is where it went wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterSyntaxError {
    pub message: String,
    pub position: usize,
}

impl FilterSyntaxError {
    pub fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position,
        }
    }
}

impl fmt::Display for FilterSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at position {}", self.message, self.position)
    }
}

impl std::error::Error for FilterSyntaxError {}

/// Result of an analysis run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub total: usize,
    pub skipped: usize,
    pub matched: Vec<LogEntry>,
}
