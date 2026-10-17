import type { PeriodKind } from "@/api/time";

// Navigation only: this moves the picker (prev, next, Today, kind) and the server validates
// the Period it is sent. The Previous Period is the server's alone; the page shows
// `report.period.previous_from/previous_to` and never computes it here.

/** Inclusive stretch of Days, as `YYYY-MM-DD`. */
export interface DayRange {
  from: string;
  to: string;
}

/** The Core week-start Setting. */
export type WeekStart = "monday" | "sunday";

const DAY_MS = 86_400_000;
const MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

// Days are computed in UTC so a daylight-saving change never shifts a calendar Day.
const toUtc = (day: string) => new Date(`${day}T00:00:00Z`);
const toDay = (date: Date) => date.toISOString().slice(0, 10);

/** `day` moved by `n` calendar Days. */
export function addDays(day: string, n: number): string {
  return toDay(new Date(toUtc(day).getTime() + n * DAY_MS));
}

/** Number of Days in the range, both ends included. */
export function dayCount({ from, to }: DayRange): number {
  return Math.round((toUtc(to).getTime() - toUtc(from).getTime()) / DAY_MS) + 1;
}

function monthRange(anchor: string, offset: number): DayRange {
  const date = toUtc(anchor);
  const first = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + offset, 1));
  const last = new Date(Date.UTC(first.getUTCFullYear(), first.getUTCMonth() + 1, 0));
  return { from: toDay(first), to: toDay(last) };
}

function weekRange(anchor: string, weekStart: WeekStart): DayRange {
  const weekday = toUtc(anchor).getUTCDay();
  const back = weekStart === "monday" ? (weekday + 6) % 7 : weekday;
  const from = addDays(anchor, -back);
  return { from, to: addDays(from, 6) };
}

/** The Period of a non-custom kind that contains `anchor`. */
export function periodFor(kind: Exclude<PeriodKind, "custom">, anchor: string, weekStart: WeekStart): DayRange {
  if (kind === "day") return { from: anchor, to: anchor };
  return kind === "week" ? weekRange(anchor, weekStart) : monthRange(anchor, 0);
}

/** The Period before (-1) or after (+1) `range`; a custom one moves by its own length. */
export function shiftPeriod(kind: PeriodKind, range: DayRange, direction: 1 | -1): DayRange {
  if (kind === "month") return monthRange(range.from, direction);
  const step = (kind === "day" ? 1 : dayCount(range)) * direction;
  return { from: addDays(range.from, step), to: addDays(range.to, step) };
}

/** The Period that holds today (a custom stretch ends today and keeps its length). */
export function jumpToToday(kind: PeriodKind, range: DayRange, today: string, weekStart: WeekStart): DayRange {
  if (kind === "custom") return { from: addDays(today, 1 - dayCount(range)), to: today };
  return periodFor(kind, today, weekStart);
}

/** True when `day` lies inside `range`. */
export function containsDay(range: DayRange, day: string): boolean {
  return range.from <= day && day <= range.to;
}

/** The range after switching kind: around today if the current range holds it, else its first Day. */
export function changeKind(kind: PeriodKind, range: DayRange, today: string, weekStart: WeekStart): DayRange {
  if (kind === "custom") return range;
  return periodFor(kind, containsDay(range, today) ? today : range.from, weekStart);
}

/** Today's local calendar Day. */
export function todayOf(now: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${String(now.getFullYear())}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

function parts(day: string) {
  const date = toUtc(day);
  return { d: date.getUTCDate(), m: MONTHS[date.getUTCMonth()]?.slice(0, 3) ?? "", y: date.getUTCFullYear(), w: WEEKDAYS[date.getUTCDay()] ?? "" };
}

/** A human label such as "12–18 Oct 2026", "October 2026" or "Fri 16 Oct 2026". */
export function periodLabel(kind: PeriodKind, range: DayRange): string {
  const a = parts(range.from);
  const b = parts(range.to);
  if (kind === "month") return `${MONTHS[toUtc(range.from).getUTCMonth()] ?? ""} ${String(a.y)}`;
  if (range.from === range.to) return `${a.w} ${String(a.d)} ${a.m} ${String(a.y)}`;
  if (a.y !== b.y) return `${String(a.d)} ${a.m} ${String(a.y)} – ${String(b.d)} ${b.m} ${String(b.y)}`;
  if (a.m !== b.m) return `${String(a.d)} ${a.m} – ${String(b.d)} ${b.m} ${String(b.y)}`;
  return `${String(a.d)}–${String(b.d)} ${b.m} ${String(b.y)}`;
}

/** The longest Period the server accepts, in Days. */
export const MAX_PERIOD_DAYS = 366;

/** `range` with its ends in order and no longer than the server accepts (the later end gives way). */
export function fitRange(range: DayRange): DayRange {
  const [from, to] = range.from <= range.to ? [range.from, range.to] : [range.to, range.from];
  return dayCount({ from, to }) > MAX_PERIOD_DAYS ? { from, to: addDays(from, MAX_PERIOD_DAYS - 1) } : { from, to };
}
