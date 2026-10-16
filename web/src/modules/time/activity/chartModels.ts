/** Shaping the server's Activity series into rows for the Recharts charts. Pure. */
import { OTHER_COLOR, seriesColor } from "@/design";
import type { ActivityDayBound, ActivityTrend } from "@/api/time";
import { eachDate } from "./dates";
import { toHours } from "./duration";

/** One Day on the first and last Activity chart; `null` where the Day has none. */
export interface BoundsPoint {
  date: string;
  first: number | null;
  last: number | null;
}

/** A point for every Day of the Period, so a Day without Activity is a gap rather than a straight line across it. */
export function boundsPoints(from: string, to: string, days: readonly ActivityDayBound[]): BoundsPoint[] {
  const byDate = new Map(days.map((day) => [day.date, day]));
  return eachDate(from, to).map((date) => {
    const day = byDate.get(date);
    return { date, first: day?.first_minute ?? null, last: day?.last_minute ?? null };
  });
}

/** One stacked series of the trend chart. */
export interface TrendSeriesInfo {
  key: string;
  label: string;
  color: string;
}

/** One week's row: `start`, how many `days` of it are in the Period, and hours under each series key. */
export type TrendRow = { start: string; days: number } & Record<string, number | string>;

/** The weekly rows in hours, and the stacked series (the busiest apps, then Other when there is any). */
export function trendChart(trend: ActivityTrend): { rows: TrendRow[]; series: TrendSeriesInfo[] } {
  const series: TrendSeriesInfo[] = trend.series.map((s, index) => ({ key: `s${String(index)}`, label: s.app, color: seriesColor(index) }));
  const hasOther = trend.other.some((seconds) => seconds > 0);
  if (hasOther) series.push({ key: "other", label: "Other apps", color: OTHER_COLOR });
  const rows = trend.weeks.map((week, index) => {
    const row: TrendRow = { start: week.start, days: week.days };
    trend.series.forEach((s, i) => {
      row[`s${String(i)}`] = toHours(s.seconds[index] ?? 0);
    });
    if (hasOther) row.other = toHours(trend.other[index] ?? 0);
    return row;
  });
  return { rows, series };
}
