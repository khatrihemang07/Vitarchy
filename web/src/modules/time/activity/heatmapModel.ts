/** Turning the server's weekday-by-hour grid into rows the heatmap draws. Pure. */

/** Which weekday a week starts on (the Core week-start setting). */
export type WeekStart = "monday" | "sunday";

const WEEKDAYS_FROM_MONDAY = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"] as const;

/** One weekday's row. */
export interface HeatmapRow {
  name: string;
  short: string;
  hours: readonly number[];
}

/** The grid's rows (Monday first on the server) in the order a week reads for `weekStart`. */
export function orderRows(grid: readonly (readonly number[])[], weekStart: WeekStart): HeatmapRow[] {
  const rows = WEEKDAYS_FROM_MONDAY.map((name, index) => ({ name, short: name.slice(0, 3), hours: grid[index] ?? [] }));
  return weekStart === "sunday" ? [...rows.slice(6), ...rows.slice(0, 6)] : rows;
}

/** The largest cell. */
export function maxCell(rows: readonly HeatmapRow[]): number {
  return Math.max(0, ...rows.flatMap((row) => row.hours));
}

/** How strongly a cell is drawn, 0 to 1. A square root keeps quiet hours visible next to the busiest. */
export function intensity(seconds: number, max: number): number {
  return max <= 0 || seconds <= 0 ? 0 : Math.sqrt(seconds / max);
}

/** The busiest cell, or null when there is no Activity. */
export function peakCell(rows: readonly HeatmapRow[]): { row: HeatmapRow; hour: number; seconds: number } | null {
  let peak: { row: HeatmapRow; hour: number; seconds: number } | null = null;
  for (const row of rows) {
    row.hours.forEach((seconds, hour) => {
      if (seconds > 0 && seconds > (peak?.seconds ?? 0)) peak = { row, hour, seconds };
    });
  }
  return peak;
}
