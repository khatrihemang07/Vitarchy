import { type ActivityOverview, type ActivityPeriod, useActivityOverview } from "@/api/time";
import { useCoreSettings } from "@/app/module-manifest";

/** One part of the Period's Activity overview. Panels asking for the same Period share one request. */
export function useActivityPart<T>(period: ActivityPeriod, select: (overview: ActivityOverview) => T) {
  return useActivityOverview(period, useCoreSettings().week_start, select);
}
