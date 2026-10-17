import type { Clock } from "./entryTimes";
import type { StatMode } from "./reportSearch";

/** What every row of one table has in common. */
export interface RowContext {
  mode: StatMode;
  clock: Clock;
  /** Days of the Period so far, for "on 4 of 6 days". */
  periodDays: number;
  /** One-Day Reports list their entries once, above the table, so bottom-level rows do not open. */
  isDay: boolean;
}
