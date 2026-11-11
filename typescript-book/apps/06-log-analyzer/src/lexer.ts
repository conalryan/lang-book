import { type CompareOp, FilterSyntaxError, type Token } from "./models.ts";

const TWO_CHAR_OPS = ["!=", "<=", ">="] as const;
const ONE_CHAR_OPS = ["=", "<", ">", "~"] as const;
const KEYWORDS = ["and", "or", "not"] as const;

const NUMBER = /^-?\d+(?:\.\d+)?/;
const IDENT = /^[A-Za-z_][\w.-]*/;

const isKeyword = (text: string): text is (typeof KEYWORDS)[number] =>
  (KEYWORDS as readonly string[]).includes(text);

/** Lazily yield tokens. A generator keeps its position between `yield`s. */
function* scan(source: string): Generator<Token> {
  let pos = 0;

  while (pos < source.length) {
    const rest = source.slice(pos);
    const char = rest[0] as string;

    if (/\s/.test(char)) {
      pos += 1;
      continue;
    }

    if (char === "(" || char === ")") {
      yield { type: "paren", text: char, pos };
      pos += 1;
      continue;
    }

    if (char === '"') {
      let text = "";
      let cursor = pos + 1;
      while (cursor < source.length && source[cursor] !== '"') {
        if (source[cursor] === "\\" && cursor + 1 < source.length) cursor += 1;
        text += source[cursor];
        cursor += 1;
      }
      if (cursor >= source.length) throw new FilterSyntaxError("unterminated string", pos);
      yield { type: "string", text, pos };
      pos = cursor + 1;
      continue;
    }

    const twoChar = TWO_CHAR_OPS.find((op) => rest.startsWith(op));
    const op: CompareOp | undefined = twoChar ?? ONE_CHAR_OPS.find((o) => o === char);
    if (op !== undefined) {
      yield { type: "op", text: op, pos };
      pos += op.length;
      continue;
    }

    const number = NUMBER.exec(rest)?.[0];
    if (number !== undefined) {
      yield { type: "number", text: number, pos };
      pos += number.length;
      continue;
    }

    const word = IDENT.exec(rest)?.[0];
    if (word !== undefined) {
      const lower = word.toLowerCase();
      yield isKeyword(lower)
        ? { type: "keyword", text: lower, pos }
        : { type: "ident", text: word, pos };
      pos += word.length;
      continue;
    }

    throw new FilterSyntaxError(`unexpected character '${char}'`, pos);
  }

  yield { type: "end", text: "", pos: source.length };
}

export function tokenize(source: string): Token[] {
  return [...scan(source)];
}
