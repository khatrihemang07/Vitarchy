import type { TimeFailure } from "@/api/time";
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
    case "no_snapshot":
      return "No Snapshot yet. Install the helper with make helper-install, then try again.";
    case "snapshot_unreadable":
      return "The Snapshot could not be read. Run make helper-run to take a fresh one.";
    case "other":
      return failure.message;
  }
}
