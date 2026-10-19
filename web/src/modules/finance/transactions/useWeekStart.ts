import { coreWithDefaults, useSettings } from "@/api/settings";

/** The Core week-start Setting; the default stands in while it loads or if it cannot. `ready` says it has answered. */
export function useWeekStart() {
  const settings = useSettings();
  return { weekStart: coreWithDefaults(settings.data?.core).week_start, ready: settings.isSuccess || settings.isError };
}
