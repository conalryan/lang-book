import { compileFilter } from "./evaluator.ts";
import { LEVELS, type Level, type LogEntry, type Summary } from "./models.ts";

const LINE = /^(?<timestamp>\S+) (?<level>DEBUG|INFO|WARN|ERROR) (?<service>\S+) (?<message>.*)$/;
const FIELD = /(\w+)=(\S+)/g;

/** Named capture groups give the regex result readable keys. */
export function parseLine(line: string): LogEntry | undefined {
  const { timestamp, level, service, message } = LINE.exec(line)?.groups ?? {};
  if (
    timestamp === undefined ||
    level === undefined ||
    service === undefined ||
    message === undefined
  ) {
    return undefined;
  }
  const fields = new Map<string, string>();
  for (const [, key, value] of message.matchAll(FIELD)) {
    if (key !== undefined && value !== undefined) fields.set(key, value);
  }
  return { raw: line, timestamp, level: level as Level, service, message, fields };
}

/** Lazy generic filter: nothing runs until the consumer pulls values. */
export function* filterLazily<T>(
  items: Iterable<T>,
  predicate: (item: T) => boolean,
): Generator<T> {
  for (const item of items) {
    if (predicate(item)) yield item;
  }
}

export function parseEntries(lines: Iterable<string>): { entries: LogEntry[]; skipped: number } {
  const entries: LogEntry[] = [];
  let skipped = 0;
  for (const line of lines) {
    if (line.trim() === "") continue;
    const entry = parseLine(line);
    if (entry === undefined) skipped += 1;
    else entries.push(entry);
  }
  return { entries, skipped };
}

/** `filter` is optional: omit it to match everything. Throws FilterSyntaxError on a bad filter. */
export function analyze(lines: Iterable<string>, filter?: string): Summary {
  const matches = filter === undefined ? () => true : compileFilter(filter);
  const { entries, skipped } = parseEntries(lines);
  return { total: entries.length, skipped, matched: [...filterLazily(entries, matches)] };
}

/** Count items by key, returning a Map in first-seen order. */
export function countBy<T, K>(items: Iterable<T>, key: (item: T) => K): Map<K, number> {
  const counts = new Map<K, number>();
  for (const item of items) {
    const k = key(item);
    counts.set(k, (counts.get(k) ?? 0) + 1);
  }
  return counts;
}

export function averageMs(entries: readonly LogEntry[]): number | undefined {
  const values = entries.map((e) => Number(e.fields.get("ms"))).filter(Number.isFinite);
  return values.length === 0 ? undefined : values.reduce((a, b) => a + b, 0) / values.length;
}

export function formatReport(summary: Summary, options: { print?: boolean } = {}): string[] {
  const { total, skipped, matched } = summary;
  const lines: string[] = [];

  if (options.print) lines.push(...matched.map((e) => e.raw));
  lines.push(`matched: ${matched.length} of ${total}`);

  if (matched.length > 0) {
    const byLevel = countBy(matched, (e) => e.level);
    lines.push(
      `levels: ${[...LEVELS]
        .reverse()
        .filter((level) => byLevel.has(level))
        .map((level) => `${level}=${byLevel.get(level)}`)
        .join(" ")}`,
    );

    const byService = [...countBy(matched, (e) => e.service)].sort(
      ([a, x], [b, y]) => y - x || a.localeCompare(b, "en"),
    );
    lines.push(`services: ${byService.map(([name, count]) => `${name}=${count}`).join(" ")}`);

    const avg = averageMs(matched);
    if (avg !== undefined) lines.push(`avg ms: ${Number(avg.toFixed(2))}`);
  }

  if (skipped > 0) lines.push(`skipped: ${skipped}`);
  return lines;
}
