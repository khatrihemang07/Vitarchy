import { useEffect } from "react";
import { useSettings } from "@/api/settings";
import { createLogger } from "@/lib/log";
import type { Clock } from "./entryTimes";
import type { WeekStart } from "./period";

const log = createLogger("time");

/** The Core week-start Setting's documented default (`WeekStart::default()` in crates/settings). */
const DEFAULT_WEEK_START: WeekStart = "monday";

/** The Core settings a Report depends on, and whether they have been read (so the Period is not computed twice). */
export function useDisplaySettings(): { weekStart: WeekStart; clock: Clock; ready: boolean } {
  const settings = useSettings();
  useEffect(() => {
    if (settings.isError) log.warn("core settings failed to load; using the default week start", { weekStart: DEFAULT_WEEK_START });
  }, [settings.isError]);
  return {
    weekStart: settings.data?.core.week_start ?? DEFAULT_WEEK_START,
    clock: settings.data?.core.clock ?? "system",
    ready: settings.isSuccess || settings.isError,
  };
}
