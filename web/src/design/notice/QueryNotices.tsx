import { type ReactNode, useState } from "react";
import { useDelayedFlag } from "@/lib/useDelayedFlag";
import { readLoadingDelay } from "./loadingDelay";
import { Notice } from "./Notice";

interface QueryNoticesProps {
  /** A TanStack query: only the parts the notices need. */
  query: { isPending: boolean; isError: boolean; error: unknown; refetch: () => Promise<unknown> };
  /** Shown once loading has lasted past the loading delay; nothing shows before that, so a fast load never flashes. */
  skeleton: ReactNode;
  /** Turns the query's error into a sentence. */
  describeError: (error: unknown) => string;
  /** Sits inside a card (no surface of its own); a full-width notice otherwise. */
  inline?: boolean;
}

/** The loading and failed-to-load states of a query, so every list handles them the same way. Failures show at once. */
export function QueryNotices({ query, skeleton, describeError, inline = false }: QueryNoticesProps) {
  const [delayMs] = useState(readLoadingDelay);
  const slow = useDelayedFlag(query.isPending && !query.isError, delayMs);
  if (query.isError) {
    return <Notice inline={inline} kind="failure" title={describeError(query.error)} onRetry={() => void query.refetch()} />;
  }
  return slow ? skeleton : null;
}
