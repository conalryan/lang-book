import {
  err,
  ok,
  type ParseError,
  Reading,
  type Result,
  type Stats,
  TemperatureLog,
  Unit,
} from "./models.ts";

const TIMESTAMP = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/;

/** Date.parse accepts Feb 30 and 24:00; round-tripping through ISO rejects them. */
function isValidTimestamp(text: string): boolean {
  if (!TIMESTAMP.test(text)) return false;
  const [year, month, day, hour, minute] = text.split(/[-T:]/).map(Number) as [
    number,
    number,
    number,
    number,
    number,
  ];
  return new Date(Date.UTC(year, month - 1, day, hour, minute)).toISOString().slice(0, 16) === text;
}

function isUnit(text: string): text is Unit {
  return (Object.values(Unit) as string[]).includes(text);
}

/** Parse `<timestamp> <value> <unit>`, e.g. `2024-01-01T08:00 21.5 C`. */
export function parseReading(line: string): Result<Reading, ParseError> {
  const fields = line.trim().split(/\s+/);
  const [timestamp, valueText, unitText] = fields;

  if (
    fields.length !== 3 ||
    timestamp === undefined ||
    valueText === undefined ||
    unitText === undefined
  ) {
    return err({ kind: "field-count", found: fields.length });
  }
  if (!isValidTimestamp(timestamp)) {
    return err({ kind: "bad-timestamp", text: timestamp });
  }
  const value = Number(valueText);
  if (!Number.isFinite(value)) {
    return err({ kind: "bad-value", text: valueText });
  }
  const unit = unitText.toUpperCase();
  if (!isUnit(unit)) {
    return err({ kind: "unknown-unit", text: unitText });
  }
  return ok(new Reading(timestamp, value, unit));
}

/** Exhaustive switch: adding a ParseError variant without handling it becomes a compile error. */
export function describeError(error: ParseError): string {
  switch (error.kind) {
    case "field-count":
      return `expected 3 fields, found ${error.found}`;
    case "bad-timestamp":
      return `invalid timestamp '${error.text}'`;
    case "bad-value":
      return `invalid temperature '${error.text}'`;
    case "unknown-unit":
      return `unknown unit '${error.text}'`;
    default: {
      const unreachable: never = error;
      throw new Error(`unhandled error ${JSON.stringify(unreachable)}`);
    }
  }
}

/** Build a log from lines; bad lines become messages instead of aborting (Result vs throw). */
export function buildLog(lines: Iterable<string>): { log: TemperatureLog; problems: string[] } {
  const log = new TemperatureLog();
  const problems: string[] = [];
  let lineNumber = 0;

  for (const line of lines) {
    lineNumber += 1;
    if (line.trim() === "") continue;
    const result = parseReading(line);
    if (result.ok) {
      log.add(result.value);
    } else {
      problems.push(`line ${lineNumber}: ${describeError(result.error)}`);
    }
  }
  return { log, problems };
}

const round2 = (value: number): string => String(Number(value.toFixed(2)));

export function formatStats(stats: Stats | undefined, latest: Reading | undefined): string[] {
  if (stats === undefined) return ["count: 0"];
  return [
    `count: ${stats.count}`,
    `min: ${round2(stats.min)} C`,
    `max: ${round2(stats.max)} C`,
    `mean: ${round2(stats.mean)} C`,
    `median: ${round2(stats.median)} C`,
    // Optional chaining + nullish coalescing: safe access with a fallback.
    `latest: ${latest?.timestamp ?? "n/a"} ${round2(latest?.celsius ?? Number.NaN)} C`,
  ];
}

/** Full report: problems first, then statistics. */
export function report(lines: Iterable<string>): string[] {
  const { log, problems } = buildLog(lines);
  return [...problems, ...formatStats(log.stats(), log.latest())];
}
