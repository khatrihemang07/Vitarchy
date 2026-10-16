import { useQuery, useQueryClient } from "@tanstack/react-query";
import { callApi } from "./callApi";
import { api } from "./client";
import { useSaveMutation } from "./saveMutation";
import type { SaveState } from "./saveStatus";
import { settingsKey } from "./settingsKeys";
import { type AllSettings, CORE_SCOPE, storedFor, withScope } from "./settingsScope";

export { CORE_SCOPE, settingsKey };
export type { AllSettings, CoreSettings, ModuleSettingsValues } from "./settingsScope";

/** Loads every scope's settings. */
export async function fetchSettings(): Promise<AllSettings> {
  return callApi(() => api.GET("/api/settings"));
}

/** Query for all settings; failures surface as `isError`, never a throw. */
export function useSettings() {
  return useQuery({ queryKey: settingsKey, queryFn: fetchSettings, retry: false });
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
