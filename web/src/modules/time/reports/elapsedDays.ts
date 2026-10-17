import { type DayRange, dayCount } from "./period";

/** Days of `range` up to and including `today`: the full count once past, 0 before it starts. */
export function elapsedDays(range: DayRange, today: string): number {
  if (today < range.from) return 0;
  return dayCount({ from: range.from, to: today < range.to ? today : range.to });
}
