import { type ActivityPeriod, type LifeLogOverview, useLifeLogOverview } from "@/api/time";
import { assignProjectColors } from "./projectColors";

/** One part of the Period's Life log overview. Panels asking for the same Period share one request. */
export function useLifeLogPart<T>(period: ActivityPeriod, select: (overview: LifeLogOverview) => T) {
  return useLifeLogOverview(period, select);
}

/** Each project's colour for the Period, the same in every panel. Undefined until the overview has loaded. */
export function useProjectColors(period: ActivityPeriod): ReadonlyMap<string, string> | undefined {
  return useLifeLogOverview(period, (overview) => assignProjectColors(overview.totals.by_project)).data;
}
