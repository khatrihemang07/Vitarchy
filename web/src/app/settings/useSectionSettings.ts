import { type AllSettings, useSaveSettings } from "@/api/settings";
import { storedFor } from "@/api/settingsScope";
import type { SaveState } from "@/api/saveStatus";
import { type SettingsSection, type SettingsValues, type SettingValue, withDefaults } from "./schema";

/**
 * A section's current values and a `save(key, value)` that persists one
 * change into the section's scope (core, or the owning Module's id).
 */
export function useSectionSettings(
  section: SettingsSection,
  settings: AllSettings,
): { values: SettingsValues; save: (key: string, value: SettingValue) => void; state: SaveState } {
  const { save, state } = useSaveSettings(section.scope);
  return { values: withDefaults(section, storedFor(settings, section.scope)), save, state };
}
