import { useEffect } from "react";
import { coreWithDefaults, useSettings } from "@/api/settings";
import { createLogger } from "@/lib/log";
import type { Clock } from "./entryTimes";
import type { WeekStart } from "./period";

const log = createLogger("time");

export interface DisplaySettings {
  weekStart: WeekStart;
  clock: Clock;
  /** Settings have answered (or failed), so the Period no longer changes under the Report. */
  ready: boolean;
  /** Settings could not load, so the defaults are in use. */
  failed: boolean;
  /** Asks the server for Settings again. */
  retry: () => void;
}

/** The Core settings a Report depends on; the shared defaults stand in while they load or if they cannot. */
export function useDisplaySettings(): DisplaySettings {
  const settings = useSettings();
  const { refetch } = settings;
  const core = coreWithDefaults(settings.data?.core);
  useEffect(() => {
    if (settings.isError) log.warn("core settings failed to load; using defaults", { weekStart: core.week_start });
  }, [settings.isError, core.week_start]);
  return {
    weekStart: core.week_start,
    clock: core.clock,
    ready: settings.isSuccess || settings.isError,
    failed: settings.isError,
    retry: () => void refetch(),
  };
}
