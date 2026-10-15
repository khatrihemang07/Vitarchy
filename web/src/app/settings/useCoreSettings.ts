import { type CoreSettings, useSettings } from "@/api/settings";
import { DEFAULT_CORE } from "./core-sections";

/** Core settings, falling back to the defaults while loading or if the server is down. */
export function useCoreSettings(): CoreSettings {
  const { data } = useSettings();
  return data?.core ?? DEFAULT_CORE;
}
