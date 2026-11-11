import { tokenize } from "./lexer.ts";
import { type Expr, FilterSyntaxError, type Token } from "./models.ts";

/**
 * Recursive-descent parser. One function per grammar rule, lowest precedence first:
 *
 *   or      := and ("or" and)*
 *   and     := not ("and" not)*
 *   not     := "not" not | primary
 *   primary := "(" or ")" | IDENT OP value
 *   value   := STRING | NUMBER | IDENT
 */
export function parse(source: string): Expr {
  const tokens = tokenize(source);
  let index = 0;

  // The lexer always ends with an "end" token, so this never runs past the array.
  const peek = (): Token => tokens[index] as Token;
  const advance = (): Token => tokens[index++] as Token;
  const describe = (token: Token): string =>
    token.type === "end" ? "end of input" : `'${token.text}'`;

  function parseOr(): Expr {
    let left = parseAnd();
    while (peek().type === "keyword" && peek().text === "or") {
      advance();
      left = { type: "or", left, right: parseAnd() };
    }
    return left;
  }

  function parseAnd(): Expr {
    let left = parseNot();
    while (peek().type === "keyword" && peek().text === "and") {
      advance();
      left = { type: "and", left, right: parseNot() };
    }
    return left;
  }

  function parseNot(): Expr {
    if (peek().type === "keyword" && peek().text === "not") {
      advance();
      return { type: "not", operand: parseNot() };
    }
    return parsePrimary();
  }

  function parsePrimary(): Expr {
    const token = advance();

    if (token.type === "paren" && token.text === "(") {
      const inner = parseOr();
      const close = advance();
      if (close.type !== "paren" || close.text !== ")") {
        throw new FilterSyntaxError(`expected ')' but found ${describe(close)}`, close.pos);
      }
      return inner;
    }

    if (token.type !== "ident") {
      throw new FilterSyntaxError(`expected a field name but found ${describe(token)}`, token.pos);
    }

    const op = advance();
    if (op.type !== "op") {
      throw new FilterSyntaxError(`expected an operator but found ${describe(op)}`, op.pos);
    }

    const value = advance();
    switch (value.type) {
      case "number":
        return { type: "compare", field: token.text, op: op.text, value: Number(value.text) };
      case "string":
      case "ident":
        return { type: "compare", field: token.text, op: op.text, value: value.text };
      default:
        throw new FilterSyntaxError(`expected a value but found ${describe(value)}`, value.pos);
    }
  }

  const expr = parseOr();
  const trailing = peek();
  if (trailing.type !== "end") {
    throw new FilterSyntaxError(`unexpected ${describe(trailing)}`, trailing.pos);
  }
  return expr;
}
