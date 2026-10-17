import { type CoreSettings, coreWithDefaults, useSettings } from "@/api/settings";

/** Core settings, with defaults for anything not loaded: while loading, if the server is down, or a field is missing. */
export function useCoreSettings(): CoreSettings {
  return coreWithDefaults(useSettings().data?.core);
}
