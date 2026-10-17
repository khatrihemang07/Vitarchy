import type { ReportGroup } from "@/api/time";
import { COMPARE, type CompareMode } from "./compareMode";
import type { StatMode } from "./reportSearch";

/** One number with its change from the chosen comparison (null when there is nothing to compare with). */
export interface Figure {
  value: number;
  change: number | null;
}

/** The Usual's per-Day average and range, unscaled. */
export interface UsualPerDay {
  avg: number;
  min: number;
  max: number;
}

/** The numbers of one table row, for the chosen stat mode and comparison. */
export interface GroupFigures {
  days: number;
  entries: number;
  /** True when the group has no time in what it is compared with. */
  isNew: boolean;
  total: Figure;
  /** The total of a one-Day Report, with the change the mode gives a Day (see `COMPARE.*.dayTotalChange`). */
  dayTotal: Figure;
  avg: Figure;
  min: Figure;
  max: Figure;
  /** The Usual's per-Day figures; null unless comparing with it and the group happened then. */
  usual: UsualPerDay | null;
}

/** Reads a group's row for the chosen stat mode and comparison; one place maps the spreads onto the columns. */
export function figuresFor(group: ReportGroup, mode: StatMode, compare: CompareMode): GroupFigures {
  const { stats } = group;
  const { change, isNew, usual } = COMPARE[compare].group(group);
  const spread = stats[mode];
  const spreadChange = change?.[mode];
  const dayChange = change ? COMPARE[compare].dayTotalChange(change) : null;
  return {
    days: stats.days,
    entries: stats.entries,
    isNew,
    total: { value: stats.total_s, change: change?.total_s ?? null },
    dayTotal: { value: stats.total_s, change: dayChange },
    avg: { value: spread.avg_s, change: spreadChange?.avg_s ?? null },
    min: { value: spread.min_s, change: spreadChange?.min_s ?? null },
    max: { value: spread.max_s, change: spreadChange?.max_s ?? null },
    usual: usual ? { avg: usual.per_day.avg_s, min: usual.per_day.min_s, max: usual.per_day.max_s } : null,
  };
}
