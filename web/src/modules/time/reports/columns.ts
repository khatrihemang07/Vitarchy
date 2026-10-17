import type { PeriodKind } from "@/api/time";
import type { CompareMode } from "./compareMode";
import { type Column, DAY_TOTAL, DAYS, ENTRIES, spreadColumns, TOTAL, USUAL, USUAL_RANGE } from "./columnDefs";
import type { StatMode } from "./reportSearch";

export type { Cell, CellContext, Column } from "./columnDefs";

/** The columns of one table and the grid classes that line header and rows up. */
export interface TableLayout {
  columns: Column[];
  /**
   * Row grid: stacked cards on narrow screens, tracks on wide ones. Written out in full so
   * Tailwind sees each class; header and rows share it so the columns always line up.
   */
  grid: string;
}

const ROW = "md:grid md:items-start";

const LONGER_GRID = `${ROW} grid-cols-3 md:grid-cols-[minmax(0,1fr)_5.5rem_4.5rem_repeat(4,6rem)]`;

/** Every layout holds its columns and its grid together, so adding a column cannot leave the grid behind. */
const LAYOUTS = {
  /** A longer Period, by stat mode. */
  longer: {
    per_day: {
      columns: [DAYS, ENTRIES, TOTAL, ...spreadColumns("per_day")],
      grid: LONGER_GRID,
    },
    per_entry: {
      columns: [DAYS, ENTRIES, TOTAL, ...spreadColumns("per_entry")],
      grid: LONGER_GRID,
    },
  },
  /** A one-Day Report drops the spread, which only repeats its total, by comparison. */
  day: {
    usual: {
      columns: [ENTRIES, DAY_TOTAL, USUAL, USUAL_RANGE],
      grid: `${ROW} grid-cols-2 md:grid-cols-[minmax(0,1fr)_4.5rem_6rem_6rem_9.5rem]`,
    },
    previous: {
      columns: [ENTRIES, DAY_TOTAL],
      grid: `${ROW} grid-cols-2 md:grid-cols-[minmax(0,1fr)_4.5rem_6rem]`,
    },
  },
} as const satisfies { longer: Record<StatMode, TableLayout>; day: Record<CompareMode, TableLayout> };

/** The layout for a Period kind, stat mode and comparison. */
export function tableLayout(kind: PeriodKind, mode: StatMode, compare: CompareMode): TableLayout {
  return kind === "day" ? LAYOUTS.day[compare] : LAYOUTS.longer[mode];
}
