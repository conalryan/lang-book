//! Recursive-descent parser.
//!
//! One method per grammar rule, lowest precedence first:
//!
//! ```text
//! or      := and ("or" and)*
//! and     := not ("and" not)*
//! not     := "not" not | primary
//! primary := "(" or ")" | IDENT OP value
//! value   := STRING | NUMBER | IDENT
//! ```

use crate::lexer::tokenize;
use crate::models::{Expr, FilterSyntaxError, Keyword, Token, TokenKind, Value};

type ParseResult<T> = Result<T, FilterSyntaxError>;

fn describe(token: &Token) -> String {
    match token.kind {
        TokenKind::End => "end of input".to_string(),
        _ => format!("'{}'", token.text()),
    }
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        // The lexer always ends with `End` and `advance` never moves past it.
        &self.tokens[self.index]
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.index].clone();
        if token.kind != TokenKind::End {
            self.index += 1;
        }
        token
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        self.peek().kind == TokenKind::Keyword(keyword)
    }

    fn parse_or(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_and()?;
        while self.at_keyword(Keyword::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_not()?;
        while self.at_keyword(Keyword::And) {
            self.advance();
            let right = self.parse_not()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> ParseResult<Expr> {
        if self.at_keyword(Keyword::Not) {
            self.advance();
            return Ok(Expr::Not(Box::new(self.parse_not()?)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> ParseResult<Expr> {
        let token = self.advance();

        if token.kind == TokenKind::LParen {
            let inner = self.parse_or()?;
            let close = self.advance();
            if close.kind != TokenKind::RParen {
                return Err(FilterSyntaxError::new(
                    format!("expected ')' but found {}", describe(&close)),
                    close.pos,
                ));
            }
            return Ok(inner);
        }

        let TokenKind::Ident(field) = token.kind else {
            return Err(FilterSyntaxError::new(
                format!("expected a field name but found {}", describe(&token)),
                token.pos,
            ));
        };

        let op_token = self.advance();
        let TokenKind::Op(op) = op_token.kind else {
            return Err(FilterSyntaxError::new(
                format!("expected an operator but found {}", describe(&op_token)),
                op_token.pos,
            ));
        };

        let value_token = self.advance();
        let value = match &value_token.kind {
            TokenKind::Number(text) => Value::Number(text.parse().unwrap_or(f64::NAN)),
            TokenKind::Str(text) | TokenKind::Ident(text) => Value::Text(text.clone()),
            _ => {
                return Err(FilterSyntaxError::new(
                    format!("expected a value but found {}", describe(&value_token)),
                    value_token.pos,
                ));
            }
        };
        Ok(Expr::Compare { field, op, value })
    }
}

/// Parse a filter expression into a syntax tree.
pub fn parse(source: &str) -> ParseResult<Expr> {
    let mut parser = Parser {
        tokens: tokenize(source)?,
        index: 0,
    };
    let expr = parser.parse_or()?;
    let trailing = parser.peek();
    if trailing.kind != TokenKind::End {
        return Err(FilterSyntaxError::new(
            format!("unexpected {}", describe(trailing)),
            trailing.pos,
        ));
    }
    Ok(expr)
}
