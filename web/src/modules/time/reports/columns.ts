import type { StatMode } from "./reportSearch";

/**
 * Grid of one summary row, wide screens only: label, Days, entries, then four
 * figures. Header and rows share it so the columns always line up. Narrow
 * screens stack each row into a card instead.
 */
export const ROW_GRID = "md:grid md:grid-cols-[minmax(0,1fr)_5.5rem_4.5rem_repeat(4,6rem)] md:items-start";

/** The figures whose wording depends on the stat mode. */
export interface FigureLabels {
  total: string;
  avg: string;
  min: string;
  max: string;
}

const LABELS: Record<StatMode, FigureLabels> = {
  per_day: { total: "Total", avg: "Avg per Day", min: "Least in a Day", max: "Most in a Day" },
  per_entry: { total: "Total", avg: "Avg per entry", min: "Shortest", max: "Longest" },
};

/** Column titles for a stat mode. */
export function figureLabels(mode: StatMode): FigureLabels {
  return LABELS[mode];
}
