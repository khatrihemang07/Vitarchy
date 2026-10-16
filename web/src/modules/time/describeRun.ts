import { failureFromCode, type TimeRun } from "@/api/time";
import { failureMessage } from "./failureMessage";
import { formatCount } from "./format";

/** The run counts worth showing, in order, with their words. */
const SHOWN_COUNTS = [
  ["inserted", "new"],
  ["updated", "updated"],
  ["deleted", "removed"],
  ["skipped_running", "still running"],
  ["skipped", "skipped"],
] as const;

function counted(counts: unknown, key: string): number {
  const value = (counts as Record<string, unknown> | null)?.[key];
  return typeof value === "number" ? value : 0;
}

/** One line on what a run did: its counts when it worked, why when it failed. */
export function describeRun(run: TimeRun): string {
  if (run.outcome !== "success") {
    return failureMessage(failureFromCode(run.error_code) ?? { kind: "other", message: "The last run failed." });
  }
  const parts = SHOWN_COUNTS.map(([key, label]) => [counted(run.counts, key), label] as const)
    .filter(([n]) => n !== 0)
    .map(([n, label]) => `${formatCount(n)} ${label}`);
  return parts.length === 0 ? "Nothing to bring in." : parts.join(", ");
}
