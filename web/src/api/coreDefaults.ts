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

/** The allowed values of each choice-type Core setting; the Settings options and `coreWithDefaults` both derive from this. */
export const CORE_CHOICES = {
  theme: ["system", "light", "dark"],
  motion: ["system", "reduced", "full"],
  week_start: ["monday", "sunday"],
  clock: ["system", "h12", "h24"],
  date_format: ["system", "day_month_year", "month_day_year", "iso"],
} as const satisfies { [K in Exclude<keyof CoreSettings, "display_name">]: readonly CoreSettings[K][] };

/** Whether `value` is acceptable for `key`: any text for the name, one of the listed choices otherwise. */
function isValid(key: keyof CoreSettings, value: unknown): boolean {
  if (key === "display_name") return typeof value === "string";
  return (CORE_CHOICES[key] as readonly unknown[]).includes(value);
}

/**
 * Core settings with every field present: each valid one the server sent, otherwise its
 * default. This also covers an older server or a damaged body, field by field.
 */
export function coreWithDefaults(data: Partial<CoreSettings> | null | undefined): CoreSettings {
  const sent: Record<string, unknown> = typeof data === "object" && data !== null ? data : {};
  const merged: Record<string, unknown> = { ...DEFAULT_CORE };
  for (const key of Object.keys(DEFAULT_CORE) as (keyof CoreSettings)[]) {
    if (isValid(key, sent[key])) merged[key] = sent[key];
  }
  return merged as unknown as CoreSettings;
}
