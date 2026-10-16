import { RequestError } from "../callApi";

/** Why a Time request failed, in terms the screen can act on. */
export type TimeFailure =
  | { kind: "invalid_token" }
  | { kind: "rate_limited"; resetsInSecs?: number }
  /** Toggl could not be reached or answered something unreadable. */
  | { kind: "toggl_unavailable" }
  /** The Vitarchy server itself could not be reached. */
  | { kind: "server_unreachable" }
  /** There is no Snapshot to Import from (the helper has not run). */
  | { kind: "no_snapshot" }
  /** The Snapshot exists but could not be read. */
  | { kind: "snapshot_unreadable" }
  | { kind: "other"; message: string };

const UNAVAILABLE_CODES = new Set(["unreachable", "unexpected", "malformed"]);

/**
 * The failure a server `code` stands for (the same codes arrive in problem
 * bodies and in a run's `error_code`), or null when the code is not one of ours.
 */
export function failureFromCode(code: unknown, resetsInSecs?: unknown): TimeFailure | null {
  if (code === "invalid_token") return { kind: "invalid_token" };
  if (code === "rate_limited") {
    return typeof resetsInSecs === "number" ? { kind: "rate_limited", resetsInSecs } : { kind: "rate_limited" };
  }
  if (code === "no_snapshot") return { kind: "no_snapshot" };
  if (code === "snapshot_unreadable") return { kind: "snapshot_unreadable" };
  if (typeof code === "string" && UNAVAILABLE_CODES.has(code)) return { kind: "toggl_unavailable" };
  return null;
}

/** Classifies any thrown error from a Time request by the problem's `code`. */
export function toTimeFailure(error: unknown): TimeFailure {
  if (!(error instanceof RequestError)) return { kind: "other", message: "Something went wrong." };
  const known = failureFromCode(error.problem?.code, error.problem?.resets_in_secs);
  if (known) return known;
  if (error.kind === "unreachable") return { kind: "server_unreachable" };
  return { kind: "other", message: error.message };
}
