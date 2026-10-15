/** What the save indicator shows for a scope of settings. */
export type SaveState =
  | { kind: "idle" }
  | { kind: "saving" }
  | { kind: "saved" }
  | { kind: "error"; message: string };

/**
 * Many saves can overlap, so the status is folded from their events rather
 * than read off one mutation. `clock` orders events: a failure is only
 * forgiven by a save that STARTED after it (that save saw the server's truth).
 */
export interface SaveStatus {
  readonly clock: number;
  /** In-flight saves: id -> the clock when it started. */
  readonly inFlight: Readonly<Record<number, number>>;
  readonly failure?: { readonly message: string; readonly clock: number };
  readonly everSaved: boolean;
}

export type SaveEvent =
  | { type: "started"; id: number }
  | { type: "succeeded"; id: number }
  | { type: "failed"; id: number; message: string };

export const initialSaveStatus: SaveStatus = { clock: 0, inFlight: {}, everSaved: false };

function without(inFlight: SaveStatus["inFlight"], id: number): SaveStatus["inFlight"] {
  return Object.fromEntries(Object.entries(inFlight).filter(([key]) => Number(key) !== id));
}

export function saveStatusReducer(status: SaveStatus, event: SaveEvent): SaveStatus {
  const clock = status.clock + 1;
  const inFlight = without(status.inFlight, event.id);
  switch (event.type) {
    case "started":
      return { ...status, clock, inFlight: { ...status.inFlight, [event.id]: clock } };
    case "failed":
      return { ...status, clock, inFlight, failure: { message: event.message, clock } };
    case "succeeded": {
      const startedAt = status.inFlight[event.id] ?? 0;
      const forgiven = status.failure !== undefined && startedAt > status.failure.clock;
      return {
        clock,
        inFlight,
        everSaved: true,
        ...(status.failure && !forgiven && { failure: status.failure }),
      };
    }
  }
}

export function saveStateOf(status: SaveStatus): SaveState {
  if (status.failure) return { kind: "error", message: status.failure.message };
  if (Object.keys(status.inFlight).length > 0) return { kind: "saving" };
  return status.everSaved ? { kind: "saved" } : { kind: "idle" };
}
