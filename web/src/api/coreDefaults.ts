import type { CoreSettings } from "./settingsScope";

/**
 * The Core settings of a fresh install. The one copy on the web side: it mirrors
 * `CoreSettings::default()` in crates/settings, and the Settings page derives its
 * field defaults from it.
 */
export const DEFAULT_CORE: CoreSettings = {
  display_name: "",
  theme: "system",
  motion: "system",
  week_start: "monday",
  clock: "system",
  date_format: "system",
};

/**
 * Core settings with every field present: each one the server sent, otherwise its
 * default. This also covers an older server or a damaged body, field by field.
 */
export function coreWithDefaults(data: Partial<CoreSettings> | null | undefined): CoreSettings {
  const sent: Record<string, unknown> = typeof data === "object" && data !== null ? data : {};
  const merged: Record<string, unknown> = { ...DEFAULT_CORE };
  for (const key of Object.keys(DEFAULT_CORE)) {
    if (typeof sent[key] === typeof merged[key]) merged[key] = sent[key];
  }
  return merged as unknown as CoreSettings;
}
