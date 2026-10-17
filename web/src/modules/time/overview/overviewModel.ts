/** The figures the Overview summaries show, derived from analysis responses. Pure. */
import type { ActivityTotals, DayTotal, LifeLogDaySplit } from "@/api/time";
import { formatSpan } from "../shared/duration";

/** A name and the seconds spent on it. */
export interface NamedSeconds {
  name: string;
  seconds: number;
}

/** The `limit` longest-used apps, longest first (the server already sorts them). */
export function topApps(totals: ActivityTotals, limit: number): NamedSeconds[] {
  return totals.apps.slice(0, limit).map((app) => ({ name: app.app, seconds: app.seconds }));
}

/** How a Period's screen time compares with the one before. */
export interface Change {
  direction: "up" | "down" | "same";
  text: string;
}

/** The average of the Days' seconds; null when there are no Days. */
function averagePerDay(days: readonly DayTotal[]): number | null {
  return days.length === 0 ? null : days.reduce((sum, day) => sum + day.seconds, 0) / days.length;
}

/**
 * "1 h more per day than the previous Period (+50%)". Compares the average per finished Day with
 * Activity, so a Period part way through (few Days so far) is not set against a whole one. Null
 * when either Period has no Day to average.
 */
export function describeChange(current: readonly DayTotal[], previous: readonly DayTotal[]): Change | null {
  const now = averagePerDay(current);
  const before = averagePerDay(previous);
  if (now === null || before === null || before <= 0) return null;
  const difference = now - before;
  if (Math.round(Math.abs(difference) / 60) === 0) return { direction: "same", text: "Same per day as the previous Period" };
  const percent = Math.round((difference / before) * 100);
  const sign = difference > 0 ? "+" : "-";
  const more = difference > 0 ? "more" : "less";
  return {
    direction: difference > 0 ? "up" : "down",
    text: `${formatSpan(Math.abs(difference))} ${more} per day than the previous Period (${sign}${String(Math.abs(percent))}%)`,
  };
}

/** Unlogged seconds, and the seconds of the Period that have happened, summed over its Days. */
export function unloggedShare(split: readonly LifeLogDaySplit[]): { unlogged: number; elapsed: number } {
  return {
    unlogged: split.reduce((sum, day) => sum + day.unlogged_seconds, 0),
    elapsed: split.reduce((sum, day) => sum + day.elapsed_seconds, 0),
  };
}
