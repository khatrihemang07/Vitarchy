import type { Report } from "@/api/time";
import type { CompareMode } from "./compareMode";
import { type TableLayout, tableLayout } from "./columns";
import type { Clock } from "./entryTimes";
import type { StatMode } from "./reportSearch";

/** What every row of one table has in common. */
export interface RowContext {
  mode: StatMode;
  compare: CompareMode;
  clock: Clock;
  /** Days of the Period so far, for "on 4 of 6 days". */
  periodDays: number;
  /** One-Day Reports list their entries once, above the table, so bottom-level rows do not open. */
  isDay: boolean;
  /** The columns and grid shared by the header and every row. */
  layout: TableLayout;
}

interface RowContextInput {
  period: Report["period"];
  mode: StatMode;
  compare: CompareMode;
  clock: Clock;
}

/** Everything a table's rows share, derived from the Report's Period and the page's choices. */
export function rowContextFor({ period, mode, compare, clock }: RowContextInput): RowContext {
  return {
    mode,
    compare,
    clock,
    periodDays: period.elapsed_days,
    isDay: period.kind === "day",
    layout: tableLayout(period.kind, mode, compare),
  };
}
