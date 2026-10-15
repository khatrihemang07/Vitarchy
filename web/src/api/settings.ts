import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useReducer, useRef } from "react";
import { api } from "./client";
import { initialSaveStatus, type SaveState, saveStateOf, saveStatusReducer } from "./saveStatus";
import type { components } from "./schema";

export type CoreSettings = components["schemas"]["CoreSettings"];
/** A Module's settings: its own JSON object. */
export type ModuleSettingsValues = Record<string, unknown>;
export interface AllSettings {
  core: CoreSettings;
  modules: Record<string, ModuleSettingsValues>;
}

/** The scope under which the core settings are stored; Modules use their id. */
export const CORE_SCOPE = "core";

export const settingsKey = ["settings"] as const;

/** A failed request carrying a message fit to show the user. */
class SettingsError extends Error {}

const UNREACHABLE = "Could not reach the server.";

function problemDetail(error: unknown): string {
  if (typeof error === "object" && error !== null && "detail" in error) {
    const { detail } = error;
    if (typeof detail === "string" && detail) return detail;
  }
  return "The server rejected this change.";
}

/** Runs a request, mapping a network failure or a problem response to a SettingsError. */
async function callApi<T>(
  request: () => Promise<{ data?: T; error?: unknown; response: { status: number } }>,
): Promise<T> {
  let result;
  try {
    result = await request();
  } catch {
    throw new SettingsError(UNREACHABLE);
  }
  if (result.data === undefined) {
    // A proxy answers 5xx (not a problem document) when the server is down.
    if (result.response.status >= 500) throw new SettingsError(UNREACHABLE);
    throw new SettingsError(problemDetail(result.error));
  }
  return result.data;
}

export async function fetchSettings(): Promise<AllSettings> {
  return callApi(() => api.GET("/api/settings"));
}

export function useSettings() {
  return useQuery({ queryKey: settingsKey, queryFn: fetchSettings, retry: false });
}

/** The stored object for a scope, or {} if nothing is saved there yet. */
function storedFor(all: AllSettings, scope: string): Record<string, unknown> {
  return scope === CORE_SCOPE ? { ...all.core } : { ...all.modules[scope] };
}

function withScope(all: AllSettings, scope: string, values: Record<string, unknown>): AllSettings {
  return scope === CORE_SCOPE
    ? { ...all, core: values as CoreSettings }
    : { ...all, modules: { ...all.modules, [scope]: values } };
}

function putScope(scope: string, values: Record<string, unknown>) {
  if (scope === CORE_SCOPE) {
    return callApi(() => api.PUT("/api/settings/core", { body: values as CoreSettings }));
  }
  return callApi(() =>
    api.PUT("/api/settings/modules/{id}", {
      params: { path: { id: scope } },
      // The generated type is `Record<string, never>`; the shape is the Module's own.
      body: values as Record<string, never>,
    }),
  );
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
  const [status, dispatch] = useReducer(saveStatusReducer, initialSaveStatus);
  const nextId = useRef(0);
  const mutationKey = ["settings", "save", scope] as const;

  const mutation = useMutation<unknown, Error, Record<string, unknown>, { id: number }>({
    mutationKey,
    scope: { id: `settings-${scope}` },
    mutationFn: (values) => putScope(scope, values),
    onMutate: () => {
      const id = nextId.current++;
      dispatch({ type: "started", id });
      return { id };
    },
    onSuccess: (_data, _values, context) => {
      dispatch({ type: "succeeded", id: context.id });
    },
    onError: (error, _values, context) => {
      if (context) dispatch({ type: "failed", id: context.id, message: error.message });
    },
    onSettled: () => {
      // This save still counts as pending here; refetch only after the last one.
      if (client.isMutating({ mutationKey }) <= 1) {
        return client.invalidateQueries({ queryKey: settingsKey });
      }
    },
  });

  const save = (key: string, value: unknown) => {
    const latest = client.getQueryData<AllSettings>(settingsKey);
    if (!latest) return;
    const next = { ...storedFor(latest, scope), [key]: value };
    // Stop a refetch already in flight from overwriting this change with older data.
    void client.cancelQueries({ queryKey: settingsKey });
    client.setQueryData(settingsKey, withScope(latest, scope, next));
    mutation.mutate(next);
  };

  return { save, state: saveStateOf(status) };
}
