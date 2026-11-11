//! Lexer: turn filter source text into tokens.

use crate::models::{CompareOp, FilterSyntaxError, Keyword, Token, TokenKind};

/// Lazily yields tokens. Implementing `Iterator` is Rust's version of a generator function:
/// the struct holds the position between calls to `next`.
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    finished: bool,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            finished: false,
        }
    }

    fn at(&self, index: usize) -> Option<char> {
        self.chars.get(index).copied()
    }

    #[allow(
        clippy::unused_self,
        clippy::unnecessary_wraps,
        reason = "keeps `next` uniform"
    )]
    fn token(&self, kind: TokenKind, pos: usize) -> Option<Result<Token, FilterSyntaxError>> {
        Some(Ok(Token { kind, pos }))
    }

    #[allow(clippy::unnecessary_wraps, reason = "keeps `next` uniform")]
    fn fail(&mut self, message: String, pos: usize) -> Option<Result<Token, FilterSyntaxError>> {
        self.finished = true;
        Some(Err(FilterSyntaxError::new(message, pos)))
    }

    /// Length of a `-?[0-9]+(\.[0-9]+)?` match starting at `start`, if any.
    fn number_len(&self, start: usize) -> Option<usize> {
        let mut i = start;
        if self.at(i) == Some('-') {
            i += 1;
        }
        let digits_start = i;
        while self.at(i).is_some_and(|c| c.is_ascii_digit()) {
            i += 1;
        }
        if i == digits_start {
            return None;
        }
        if self.at(i) == Some('.') && self.at(i + 1).is_some_and(|c| c.is_ascii_digit()) {
            i += 1;
            while self.at(i).is_some_and(|c| c.is_ascii_digit()) {
                i += 1;
            }
        }
        Some(i - start)
    }

    fn operator(&self) -> Option<(CompareOp, usize)> {
        let first = self.at(self.pos)?;
        let second = self.at(self.pos + 1);
        match (first, second) {
            ('!', Some('=')) => Some((CompareOp::Ne, 2)),
            ('<', Some('=')) => Some((CompareOp::Le, 2)),
            ('>', Some('=')) => Some((CompareOp::Ge, 2)),
            ('=', _) => Some((CompareOp::Eq, 1)),
            ('<', _) => Some((CompareOp::Lt, 1)),
            ('>', _) => Some((CompareOp::Gt, 1)),
            ('~', _) => Some((CompareOp::Contains, 1)),
            _ => None,
        }
    }
}

impl Iterator for Lexer {
    type Item = Result<Token, FilterSyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        while self.at(self.pos).is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
        let start = self.pos;
        let Some(c) = self.at(start) else {
            self.finished = true;
            return self.token(TokenKind::End, start);
        };

        if c == '(' || c == ')' {
            self.pos += 1;
            return self.token(
                if c == '(' {
                    TokenKind::LParen
                } else {
                    TokenKind::RParen
                },
                start,
            );
        }

        if c == '"' {
            let mut text = String::new();
            let mut cursor = start + 1;
            while let Some(ch) = self.at(cursor) {
                if ch == '"' {
                    break;
                }
                // A backslash escapes the next character.
                let take = if ch == '\\' && self.at(cursor + 1).is_some() {
                    cursor + 1
                } else {
                    cursor
                };
                text.push(self.chars[take]);
                cursor = take + 1;
            }
            if self.at(cursor).is_none() {
                return self.fail("unterminated string".to_string(), start);
            }
            self.pos = cursor + 1;
            return self.token(TokenKind::Str(text), start);
        }

        if let Some((op, len)) = self.operator() {
            self.pos += len;
            return self.token(TokenKind::Op(op), start);
        }

        if let Some(len) = self.number_len(start) {
            self.pos += len;
            let text: String = self.chars[start..start + len].iter().collect();
            return self.token(TokenKind::Number(text), start);
        }

        if c.is_ascii_alphabetic() || c == '_' {
            let mut end = start;
            while self
                .at(end)
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || "_.-".contains(ch))
            {
                end += 1;
            }
            self.pos = end;
            let word: String = self.chars[start..end].iter().collect();
            let kind = match word.to_lowercase().as_str() {
                "and" => TokenKind::Keyword(Keyword::And),
                "or" => TokenKind::Keyword(Keyword::Or),
                "not" => TokenKind::Keyword(Keyword::Not),
                _ => TokenKind::Ident(word),
            };
            return self.token(kind, start);
        }

        self.fail(format!("unexpected character '{c}'"), start)
    }
}

/// All tokens, ending with an `End` token. Collecting an iterator of `Result`s into one
/// `Result<Vec<_>, _>` stops at the first error.
pub fn tokenize(source: &str) -> Result<Vec<Token>, FilterSyntaxError> {
    Lexer::new(source).collect()
}
