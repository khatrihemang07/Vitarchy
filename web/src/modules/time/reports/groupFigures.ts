import type { ReportGroup } from "@/api/time";
import type { StatMode } from "./reportSearch";

/** One number with its change from the Previous Period (null when there is nothing to compare with). */
export interface Figure {
  value: number;
  change: number | null;
}

/** The numbers of one table row, for the chosen stat mode. */
export interface GroupFigures {
  days: number;
  entries: number;
  isNew: boolean;
  total: Figure;
  avg: Figure;
  min: Figure;
  max: Figure;
}

/** Reads a group's row for the chosen stat mode; one place maps the two spreads onto the same columns. */
export function figuresFor(group: ReportGroup, mode: StatMode): GroupFigures {
  const { stats, change } = group;
  const spread = stats[mode];
  const spreadChange = change?.[mode];
  return {
    days: stats.days,
    entries: stats.entries,
    isNew: group.is_new,
    total: { value: stats.total_s, change: change?.total_s ?? null },
    avg: { value: spread.avg_s, change: spreadChange?.avg_s ?? null },
    min: { value: spread.min_s, change: spreadChange?.min_s ?? null },
    max: { value: spread.max_s, change: spreadChange?.max_s ?? null },
  };
}
