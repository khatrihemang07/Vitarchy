/** The figures the Overview summaries show, derived from analysis responses. Pure. */
import type { ActivityTotals, LifeLogDaySplit } from "@/api/time";
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

/**
 * The `limit` longest-used Contexts across apps. A Context that only repeats its app's name (any
 * app that is neither an editor nor a browser) is left out, since the apps list already shows it.
 */
export function topContexts(totals: ActivityTotals, limit: number): NamedSeconds[] {
  const byName = new Map<string, number>();
  for (const app of totals.apps) {
    for (const context of app.contexts) {
      if (context.name !== app.app) byName.set(context.name, (byName.get(context.name) ?? 0) + context.seconds);
    }
  }
  return [...byName]
    .map(([name, seconds]) => ({ name, seconds }))
    .sort((a, b) => b.seconds - a.seconds || a.name.localeCompare(b.name))
    .slice(0, limit);
}

/** How a Period's screen time compares with the one before. */
export interface Change {
  direction: "up" | "down" | "same";
  text: string;
}

/** "1 h more than the previous Period (+50%)"; null when the previous Period had nothing to compare with. */
export function describeChange(current: number, previous: number): Change | null {
  if (previous <= 0) return null;
  const difference = current - previous;
  if (Math.round(Math.abs(difference) / 60) === 0) return { direction: "same", text: "Same as the previous Period" };
  const percent = Math.round((difference / previous) * 100);
  const sign = difference > 0 ? "+" : "-";
  const more = difference > 0 ? "more" : "less";
  return {
    direction: difference > 0 ? "up" : "down",
    text: `${formatSpan(Math.abs(difference))} ${more} than the previous Period (${sign}${String(Math.abs(percent))}%)`,
  };
}

/** Unlogged seconds, and the seconds of the Period that have happened, summed over its Days. */
export function unloggedShare(split: readonly LifeLogDaySplit[]): { unlogged: number; elapsed: number } {
  return {
    unlogged: split.reduce((sum, day) => sum + day.unlogged_seconds, 0),
    elapsed: split.reduce((sum, day) => sum + day.elapsed_seconds, 0),
  };
}
