/** Shaping the server's daily split into rows for the stacked Recharts bars. Pure. */
import type { LifeLogDaySplit } from "@/api/time";
import { OTHER_COLOR } from "@/design";
import { toHours } from "../shared/duration";

/** One stacked series: a project, or the unlogged remainder. */
export interface SplitSeries {
  key: string;
  label: string;
  color: string;
}

/** One Day's row: hours under each series key (for the bars) and exact seconds (for the words). */
export type SplitRow = { date: string; seconds: Record<string, number> } & Record<string, number | string | Record<string, number>>;

const UNLOGGED_KEY = "unlogged";

/** Colour of the unlogged remainder. */
export const UNLOGGED_COLOR = "var(--chart-unlogged)";

/** The series (projects in the given order, then "Unlogged") and a row for every Day. */
export function splitChart(
  split: readonly LifeLogDaySplit[],
  projects: readonly { name: string }[],
  colors: ReadonlyMap<string, string>,
): { rows: SplitRow[]; series: SplitSeries[] } {
  const projectSeries: SplitSeries[] = projects.map((p, index) => ({ key: `p${String(index)}`, label: p.name, color: colors.get(p.name) ?? OTHER_COLOR }));
  const series: SplitSeries[] = [...projectSeries, { key: UNLOGGED_KEY, label: "Unlogged", color: UNLOGGED_COLOR }];
  // Only projects are looked up by name: a project called "Unlogged" must not claim the remainder's key.
  const keyOf = new Map(projectSeries.map((s) => [s.label, s.key]));
  const rows = split.map((day) => {
    const seconds: Record<string, number> = Object.fromEntries(series.map((s) => [s.key, 0]));
    for (const part of day.by_project) {
      const key = keyOf.get(part.project);
      if (key) seconds[key] = part.seconds;
    }
    seconds[UNLOGGED_KEY] = day.unlogged_seconds;
    const row: SplitRow = { date: day.date, seconds };
    for (const s of series) row[s.key] = toHours(seconds[s.key] ?? 0);
    return row;
  });
  return { rows, series };
}

const HOURS_STEP = 6;
const HOURS_PER_DAY = 24;

/** The hours axis: 0 to 24 in steps of six, growing by whole steps for a Day that overlapping entries push past 24. */
export function hoursAxis(rows: readonly SplitRow[]): { top: number; ticks: number[] } {
  const tallest = Math.max(0, ...rows.map((row) => Object.values(row).reduce<number>((sum, value) => sum + (typeof value === "number" ? value : 0), 0)));
  const top = Math.max(HOURS_PER_DAY, Math.ceil(tallest / HOURS_STEP) * HOURS_STEP);
  return { top, ticks: Array.from({ length: top / HOURS_STEP + 1 }, (_, i) => i * HOURS_STEP) };
}
