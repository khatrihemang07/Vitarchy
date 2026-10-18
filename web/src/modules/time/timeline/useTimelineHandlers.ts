import { useMemo } from "react";
import type { PeriodKind } from "@/api/time";
import { changeKind, type DayRange, fitRange, jumpToToday, shiftPeriod, type WeekStart } from "../reports/period";
import { type TimelineSearch, withApp, withPeriod } from "./timelineSearch";

interface HandlerInput {
  search: TimelineSearch;
  range: DayRange;
  today: string;
  weekStart: WeekStart;
  update: (next: TimelineSearch) => void;
}

/** Every change the controls can make, expressed as a new URL state. */
export function useTimelineHandlers({ search, range, today, weekStart, update }: HandlerInput) {
  return useMemo(
    () => ({
      onKind: (kind: PeriodKind) => { update(withPeriod(search, kind, changeKind(kind, range, today, weekStart))); },
      onShift: (direction: 1 | -1) => { update(withPeriod(search, search.kind, shiftPeriod(search.kind, range, direction))); },
      onToday: () => { update(withPeriod(search, search.kind, jumpToToday(search.kind, range, today, weekStart))); },
      onCustom: (next: DayRange) => { update(withPeriod(search, "custom", fitRange(next))); },
      onApp: (app: string | undefined) => { update(withApp(search, app)); },
    }),
    [search, range, today, weekStart, update],
  );
}
