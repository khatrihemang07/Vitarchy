import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useReducer, useRef } from "react";
import { createLogger } from "@/lib/log";
import { callApi, RequestError } from "./callApi";
import { api } from "./client";
import { initialSaveStatus, type SaveState, saveStateOf, saveStatusReducer } from "./saveStatus";
import { saveMutationKey, settingsKey } from "./settingsKeys";
import { CORE_SCOPE, type CoreSettings } from "./settingsScope";

const log = createLogger("settings");

/** One change to save: the key (safe to log) and the scope's full next values (not). */
export interface SaveRequest {
  key: string;
  values: Record<string, unknown>;
}

/** Replaces one scope's whole settings object on the server. */
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
 * The mutation behind `useSaveSettings`: sends a scope's values, tracks the
 * save status, and refetches once the last pending save settles.
 */
export function useSaveMutation(scope: string): {
  mutate: (request: SaveRequest) => void;
  state: SaveState;
} {
  const client = useQueryClient();
  const [status, dispatch] = useReducer(saveStatusReducer, initialSaveStatus);
  const nextId = useRef(0);
  const mutationKey = saveMutationKey(scope);

  const mutation = useMutation<unknown, Error, SaveRequest, { id: number }>({
    mutationKey,
    scope: { id: `settings-${scope}` },
    mutationFn: ({ values }) => putScope(scope, values),
    onMutate: () => {
      const id = nextId.current++;
      dispatch({ type: "started", id });
      return { id };
    },
    onSuccess: (_data, _values, context) => {
      dispatch({ type: "succeeded", id: context.id });
    },
    onError: (error, { key }, context) => {
      // Never the message: it can quote what the person entered.
      const failure = error instanceof RequestError ? { kind: error.kind, status: error.status } : {};
      log.warn("save failed", { scope, key, ...failure });
      if (context) dispatch({ type: "failed", id: context.id, message: error.message });
    },
    onSettled: (_data, error) => {
      // This save still counts as pending here; refetch only after the last one.
      if (client.isMutating({ mutationKey }) <= 1) {
        if (error) log.debug("refetching settings to match the server", { scope });
        return client.invalidateQueries({ queryKey: settingsKey });
      }
    },
  });

  return { mutate: mutation.mutate, state: saveStateOf(status) };
}
