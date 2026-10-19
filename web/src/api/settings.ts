import { queryOptions, useQuery, useQueryClient } from "@tanstack/react-query";
import { callApi } from "./callApi";
import { CORE_CHOICES, coreWithDefaults, DEFAULT_CORE } from "./coreDefaults";
import { api } from "./client";
import { useSaveMutation } from "./saveMutation";
import type { SaveState } from "./saveStatus";
import { settingsKey } from "./settingsKeys";
import { type AllSettings, CORE_SCOPE, storedFor, withScope } from "./settingsScope";

export { CORE_CHOICES, coreWithDefaults, CORE_SCOPE, DEFAULT_CORE, settingsKey };
export type { AllSettings, CoreSettings, ModuleSettingsValues } from "./settingsScope";

/** Loads every scope's settings. */
export async function fetchSettings(): Promise<AllSettings> {
  return callApi(() => api.GET("/api/settings"));
}

/** How to load all settings; shared by the hook and by route loaders that warm the cache. */
export const settingsOptions = queryOptions({ queryKey: settingsKey, queryFn: fetchSettings, retry: false });

/** Query for all settings; failures surface as `isError`, never a throw. */
export function useSettings() {
  return useQuery(settingsOptions);
}

/**
 * Saves single settings in one scope (`CORE_SCOPE` or a Module id), applying
 * each at once and re-syncing with the server afterwards.
 *
 * - Each change is built from the LATEST cache when `save` is called, so
 *   rapid changes and stale closures never drop one another's keys.
 * - Saves in a scope run one after another (a mutation `scope`), so the
 *   server sees them in the order the user made them.
 * - A failed save does not restore an old snapshot (which could erase later
 *   changes); the cache is refetched, once the last pending save settles.
 */
export function useSaveSettings(scope: string): {
  save: (key: string, value: unknown) => void;
  state: SaveState;
} {
  const client = useQueryClient();
  const { mutate, state } = useSaveMutation(scope);

  const save = (key: string, value: unknown) => {
    const latest = client.getQueryData<AllSettings>(settingsKey);
    if (!latest) return;
    const next = { ...storedFor(latest, scope), [key]: value };
    // Stop a refetch already in flight from overwriting this change with older data.
    void client.cancelQueries({ queryKey: settingsKey });
    client.setQueryData(settingsKey, withScope(latest, scope, next));
    mutate({ key, values: next });
  };

  return { save, state };
}
