import { type TimeFailure, toTimeFailure } from "@/api/time";
import { formatDuration } from "./format";

/** Words for a failed Time request, fit to show the person. */
export function failureMessage(failure: TimeFailure): string {
  switch (failure.kind) {
    case "invalid_token":
      return "Toggl rejected this API token. Check it and connect again.";
    case "rate_limited":
      return failure.resetsInSecs === undefined
        ? "Toggl's request limit was reached. Try again later."
        : `Toggl's request limit was reached. It resets in ${formatDuration(failure.resetsInSecs)}.`;
    case "toggl_unavailable":
      return "Could not reach Toggl. Check your connection and try again.";
    case "server_unreachable":
      return "Could not reach the Vitarchy server.";
    case "sync_in_progress":
      return "A Sync is already running. Wait for it to finish.";
    case "connection_changed":
      return "Toggl was connected again while syncing, so nothing was saved. Sync again.";
    case "no_snapshot":
      return "No Snapshot yet. Install the helper with make helper-install, then try again.";
    case "snapshot_unreadable":
      return "The Snapshot could not be read. Run make helper-run to take a fresh one.";
    case "other":
      return failure.message;
  }
}

/** Words for whatever a failed Time query or mutation threw. */
export function errorMessage(error: unknown): string {
  return failureMessage(toTimeFailure(error));
}
