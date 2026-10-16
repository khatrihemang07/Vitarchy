import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi, RequestError } from "../callApi";
import { api } from "../client";
import type { ActivityPeriod } from "./activityAnalysis";
import { routinesKey } from "./keys";
import type { RoutineDraft } from "./types";

const log = createLogger("time.routines");

/** Recurring sequences in the Period that are not saved as Routines yet. */
export function useRoutineSuggestions(period: ActivityPeriod) {
  return useQuery({
    queryKey: routinesKey("suggestions", period),
    queryFn: () => callApi(() => api.GET("/api/time/routines/suggestions", { params: { query: period } })),
    select: (body) => body.suggestions,
    retry: false,
  });
}

/** The saved Routines. */
export function useRoutines() {
  return useQuery({
    queryKey: routinesKey("list"),
    queryFn: () => callApi(() => api.GET("/api/time/routines")),
    select: (body) => body.routines,
    retry: false,
  });
}

/** How one Routine went over the Period. */
export function useRoutineOccurrences(id: number, period: ActivityPeriod) {
  return useQuery({
    queryKey: routinesKey("occurrences", id, period),
    queryFn: () =>
      callApi(() => api.GET("/api/time/routines/{id}/occurrences", { params: { path: { id }, query: period } })),
    retry: false,
  });
}

/** Saves a Routine: a new one, or a replacement of the one with `id`. Refreshes every Routine query. */
export function useSaveRoutine() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ id, draft }: { id?: number | undefined; draft: RoutineDraft }) =>
      id === undefined
        ? callApi(() => api.POST("/api/time/routines", { body: draft }))
        : callApi(() => api.PUT("/api/time/routines/{id}", { params: { path: { id } }, body: draft })),
    onSuccess: (saved) => {
      log.info("routine saved", { routineId: saved.id });
      return client.invalidateQueries({ queryKey: routinesKey() });
    },
    onError: (error) => {
      log.warn("routine save failed", { status: error instanceof RequestError ? error.status : undefined });
    },
  });
}

/** Deletes a Routine. Refreshes every Routine query. */
export function useDeleteRoutine() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => callApi(() => api.DELETE("/api/time/routines/{id}", { params: { path: { id } } })),
    onSuccess: (_, id) => {
      log.info("routine deleted", { routineId: id });
      return client.invalidateQueries({ queryKey: routinesKey() });
    },
  });
}
