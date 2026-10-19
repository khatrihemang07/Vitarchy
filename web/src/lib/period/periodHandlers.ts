import { changeKind, type DayRange, fitRange, jumpToToday, type PeriodKind, shiftPeriod, type WeekStart } from "./period";
import { type PeriodSearch, withRange } from "./periodSearch";

interface PeriodHandlerInput<T extends PeriodSearch> {
  search: T;
  /** The Days the state covers now. */
  range: DayRange;
  today: string;
  weekStart: WeekStart;
  update: (next: T) => void;
}

/** What the Period picker can do, each as a new URL state (the rest of the state is kept). */
export function periodHandlers<T extends PeriodSearch>({ search, range, today, weekStart, update }: PeriodHandlerInput<T>) {
  return {
    onKind: (kind: PeriodKind) => { update(withRange(search, kind, changeKind(kind, range, today, weekStart))); },
    onShift: (direction: 1 | -1) => { update(withRange(search, search.kind, shiftPeriod(search.kind, range, direction))); },
    onToday: () => { update(withRange(search, search.kind, jumpToToday(search.kind, range, today, weekStart))); },
    onCustom: (next: DayRange) => { update(withRange(search, "custom", fitRange(next))); },
  };
}
