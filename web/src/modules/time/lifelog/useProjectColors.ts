import { type ActivityPeriod, type LifeLogOverview, useLifeLogOverview } from "@/api/time";
import { assignProjectColors } from "./projectColors";

/** Defined once so the query memoizes the result: the colours are computed once per response, not per render. */
const selectProjectColors = (overview: LifeLogOverview) => assignProjectColors(overview.totals.by_project);

/** Each project's colour for the Period, the same in every panel. Undefined until the overview has loaded. */
export function useProjectColors(period: ActivityPeriod): ReadonlyMap<string, string> | undefined {
  return useLifeLogOverview(period, selectProjectColors).data;
}
