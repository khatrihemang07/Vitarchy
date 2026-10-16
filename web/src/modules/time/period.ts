/** The Period the person is looking at: kinds, ranges, moving and naming. Pure; dates are local `YYYY-MM-DD`. */

/** How a Period is chosen. */
export type PeriodKind = "day" | "week" | "month" | "custom";
/** Which weekday a week starts on (the Core week-start Setting). */
export type WeekStart = "monday" | "sunday";

/** A chosen Period: its kind and its first and last local Day (inclusive). */
export interface PeriodSelection {
  kind: PeriodKind;
  from: string;
  to: string;
}

/** The kinds in the order the picker shows them. */
export const PERIOD_KINDS: readonly { value: PeriodKind; label: string }[] = [
  { value: "day", label: "Day" },
  { value: "week", label: "Week" },
  { value: "month", label: "Month" },
  { value: "custom", label: "Custom" },
];

/** The longest Period the server accepts, in Days. Mirrors `MAX_DAYS` in crates/modules/time/src/analysis/period.rs (scripts/test-guards.sh keeps them equal). */
export const MAX_PERIOD_DAYS = 400;

const DAY_MS = 86_400_000;
const FIRST_WEEKDAY: Record<WeekStart, number> = { monday: 1, sunday: 0 };
const ISO = /^\d{4}-\d{2}-\d{2}$/;

function toMs(iso: string): number {
  const [year, month, day] = iso.split("-").map(Number);
  return Date.UTC(year ?? 1970, (month ?? 1) - 1, day ?? 1);
}

function toIso(ms: number): string {
  return new Date(ms).toISOString().slice(0, 10);
}

/** Whether `text` is a real calendar date written `YYYY-MM-DD`. */
export function isIsoDate(text: string): boolean {
  return ISO.test(text) && toIso(toMs(text)) === text;
}

/** How many Days `from..=to` covers, both ends included. */
export function spanDays(from: string, to: string): number {
  return (toMs(to) - toMs(from)) / DAY_MS + 1;
}

/** Today's date in the person's own time zone, as `YYYY-MM-DD`. */
export function localIsoDate(now: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${String(now.getFullYear())}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

function monthRange(anchor: string, monthOffset: number): { from: string; to: string } {
  const date = new Date(toMs(anchor));
  const year = date.getUTCFullYear();
  const month = date.getUTCMonth() + monthOffset;
  return { from: toIso(Date.UTC(year, month, 1)), to: toIso(Date.UTC(year, month + 1, 0)) };
}

/** The Period of `kind` that contains `anchor`. A custom range starts as just the anchor day. */
export function selectionFor(kind: PeriodKind, anchor: string, weekStart: WeekStart): PeriodSelection {
  switch (kind) {
    case "week": {
      const back = (new Date(toMs(anchor)).getUTCDay() - FIRST_WEEKDAY[weekStart] + 7) % 7;
      const from = toMs(anchor) - back * DAY_MS;
      return { kind, from: toIso(from), to: toIso(from + 6 * DAY_MS) };
    }
    case "month":
      return { kind, ...monthRange(anchor, 0) };
    case "day":
    case "custom":
      return { kind, from: anchor, to: anchor };
  }
}

/** Changes the kind, keeping what the person was looking at (a custom range keeps its dates). */
export function switchKind(current: PeriodSelection, kind: PeriodKind, weekStart: WeekStart): PeriodSelection {
  return kind === "custom" ? { kind, from: current.from, to: current.to } : selectionFor(kind, current.from, weekStart);
}

/** The Period one step later (`1`) or earlier (`-1`): a day, week or month, or a custom range's own length. */
export function shiftPeriod(selection: PeriodSelection, direction: 1 | -1): PeriodSelection {
  const { kind, from, to } = selection;
  if (kind === "month") return { kind, ...monthRange(from, direction) };
  const days = kind === "day" ? 1 : kind === "week" ? 7 : (toMs(to) - toMs(from)) / DAY_MS + 1;
  const offset = direction * days * DAY_MS;
  return { kind, from: toIso(toMs(from) + offset), to: toIso(toMs(to) + offset) };
}

/** The Period right before this one, for comparing. */
export function previousPeriod(selection: PeriodSelection): PeriodSelection {
  return shiftPeriod(selection, -1);
}

/** Whether the next Period may be shown: a day, week or month once it has begun; a custom range only if it ends by today. */
export function canGoNext(selection: PeriodSelection, today: string): boolean {
  const next = shiftPeriod(selection, 1);
  return selection.kind === "custom" ? next.to <= today : next.from <= today;
}

/** A readable name such as "Oct 14, 2026", "October 2026" or "Oct 12 – 18, 2026". */
export function labelPeriod(selection: PeriodSelection, locale?: string): string {
  const options = { timeZone: "UTC" } as const;
  const from = new Date(toMs(selection.from));
  const to = new Date(toMs(selection.to));
  if (selection.kind === "month") {
    return new Intl.DateTimeFormat(locale, { ...options, month: "long", year: "numeric" }).format(from);
  }
  const format = new Intl.DateTimeFormat(locale, { ...options, dateStyle: "medium" });
  return selection.from === selection.to ? format.format(from) : format.formatRange(from, to);
}
