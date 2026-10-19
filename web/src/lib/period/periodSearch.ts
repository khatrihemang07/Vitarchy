import { type DayRange, fitRange, type PeriodKind, periodFor, type WeekStart } from "./period";

/** The part of a page's URL that chooses a Period; shared by every page that has a Period picker. */
export interface PeriodSearch {
  kind: PeriodKind;
  /** Any Day inside a Day, week or month Period; today when absent. */
  date?: string;
  /** Custom Periods only. */
  from?: string;
  /** Custom Periods only. */
  to?: string;
}

const KINDS: readonly PeriodKind[] = ["day", "week", "month", "custom"];

/** A `YYYY-MM-DD` that is a real calendar Day, or undefined. */
export function validDay(value: unknown): string | undefined {
  if (typeof value !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return undefined;
  const date = new Date(`${value}T00:00:00Z`);
  return Number.isNaN(date.getTime()) || date.toISOString().slice(0, 10) !== value ? undefined : value;
}

function customRange(raw: Record<string, unknown>): DayRange | undefined {
  const a = validDay(raw.from);
  const b = validDay(raw.to);
  if (!a || !b) return undefined;
  return fitRange({ from: a, to: b });
}

/** The Period the URL names; anything unusable falls back to `defaultKind` (the URL is user-editable). */
export function parsePeriodSearch(raw: Record<string, unknown>, defaultKind: Exclude<PeriodKind, "custom">): PeriodSearch {
  const range = raw.kind === "custom" ? customRange(raw) : undefined;
  const kind = range ? "custom" : (KINDS.find((k) => k === raw.kind) ?? defaultKind);
  const date = kind === "custom" ? undefined : validDay(raw.date);
  return {
    kind: kind === "custom" && !range ? defaultKind : kind,
    ...(date && { date }),
    ...(range && range),
  };
}

/** The URL form of a Period: the default kind is left out. */
export function periodParams(search: PeriodSearch, defaultKind: PeriodKind): Record<string, unknown> {
  return {
    ...(search.kind !== defaultKind && { kind: search.kind }),
    ...(search.date && { date: search.date }),
    ...(search.from && { from: search.from }),
    ...(search.to && { to: search.to }),
  };
}

/** The Days the state covers; relative to `today` when the URL names no Day. */
export function rangeOf(search: PeriodSearch, today: string, weekStart: WeekStart): DayRange {
  if (search.kind === "custom" && search.from && search.to) return { from: search.from, to: search.to };
  return periodFor(search.kind === "custom" ? "week" : search.kind, search.date ?? today, weekStart);
}

/** The state after moving to `range`; a custom stretch keeps both ends, others one anchor Day. */
export function withRange<T extends PeriodSearch>(search: T, kind: PeriodKind, range: DayRange): T {
  const next: T = { ...search, kind };
  delete next.date;
  delete next.from;
  delete next.to;
  return kind === "custom" ? { ...next, from: range.from, to: range.to } : { ...next, date: range.from };
}
