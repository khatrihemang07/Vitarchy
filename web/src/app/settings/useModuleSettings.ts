import { useSaveSettings, useSettings } from "@/api/settings";
import { type SettingsSection, type ValuesOf, withDefaults } from "./schema";

/**
 * For a Module: its own settings, read with the declared defaults applied
 * (missing or invalid stored values fall back), and a `save(key, value)` that
 * persists one change under the Module id.
 *
 * Declare `settings` `as const` to get each key's own type.
 */
export function useModuleSettings<S extends Pick<SettingsSection, "fields">>(
  moduleId: string,
  settings: S,
): {
  values: ValuesOf<S>;
  save: <K extends keyof ValuesOf<S> & string>(key: K, value: ValuesOf<S>[K]) => void;
} {
  const { data } = useSettings();
  const { save } = useSaveSettings(moduleId);
  // `withDefaults` guarantees the declared shape, which TypeScript cannot see across the field union.
  const values = withDefaults(settings, data?.modules[moduleId]) as ValuesOf<S>;
  return { values, save };
}
