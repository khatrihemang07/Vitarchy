import { useMemo } from "react";
import type { Grouping, PeriodKind } from "@/api/time";
import { changeKind, type DayRange, fitRange, jumpToToday, shiftPeriod, type WeekStart } from "./period";
import { clearFilters, type FacetKey, withFacet, withGroups, withRange } from "./reportActions";
import type { ReportSearch, StatMode } from "./reportSearch";

interface HandlerInput {
  search: ReportSearch;
  range: DayRange;
  today: string;
  weekStart: WeekStart;
  update: (next: ReportSearch) => void;
}

/** Every change the controls can make, expressed as a new URL state. */
export function useReportHandlers({ search, range, today, weekStart, update }: HandlerInput) {
  return useMemo(
    () => ({
      onKind: (kind: PeriodKind) => { update(withRange(search, kind, changeKind(kind, range, today, weekStart))); },
      onShift: (direction: 1 | -1) => { update(withRange(search, search.kind, shiftPeriod(search.kind, range, direction))); },
      onToday: () => { update(withRange(search, search.kind, jumpToToday(search.kind, range, today, weekStart))); },
      onCustom: (next: DayRange) => { update(withRange(search, "custom", fitRange(next))); },
      onFacet: (key: FacetKey, values: string[]) => { update(withFacet(search, key, values)); },
      onSearch: (q: string) => { update({ ...search, q }); },
      onGroups: (group1: Grouping, group2: Grouping | undefined) => { update(withGroups(search, group1, group2)); },
      onMode: (mode: StatMode) => { update({ ...search, mode }); },
      onClear: () => { update(clearFilters(search)); },
    }),
    [search, range, today, weekStart, update],
  );
}
