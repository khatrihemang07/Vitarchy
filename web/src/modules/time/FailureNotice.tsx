import type { TimeFailure } from "@/api/time";
import { failureMessage } from "./failureMessage";

/** An inline, announced explanation of a failed request. */
export function FailureNotice({ failure }: { failure: TimeFailure }) {
  return (
    <p role="alert" className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
      {failureMessage(failure)}
    </p>
  );
}
