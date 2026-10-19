import { useState } from "react";
import { useDelayedFlag } from "@/lib/useDelayedFlag";
import { FadeUp } from "../motion/FadeUp";
import { readLoadingDelay } from "./loadingDelay";
import { Notice } from "./Notice";

interface QueryNoticesProps {
  /** A TanStack query: only the parts the notices need. */
  query: { isPending: boolean; isError: boolean; error: unknown; refetch: () => Promise<unknown> };
  /** The loading notice's message ("Loading report…"); it shows once loading has lasted past the loading delay, so a fast load never flashes. */
  loadingTitle: string;
  /** Turns the query's error into a sentence. */
  describeError: (error: unknown) => string;
  /** Sits inside a card (no surface of its own); a full-width notice otherwise. */
  inline?: boolean;
  /** Place in the page's entrance sequence. The notice brings its own `FadeUp`, so nothing (not even an empty wrapper) sits in the page while the loading delay runs. */
  stagger?: number;
}

/** The loading and failed-to-load notices of a query, so every page handles them the same way. Failures show at once. */
export function QueryNotices({ query, loadingTitle, describeError, inline = false, stagger }: QueryNoticesProps) {
  const [delayMs] = useState(readLoadingDelay);
  const slow = useDelayedFlag(query.isPending && !query.isError, delayMs);
  const notice = query.isError ? (
    <Notice inline={inline} kind="failure" title={describeError(query.error)} onRetry={() => void query.refetch()} />
  ) : slow ? (
    <Notice inline={inline} kind="loading" title={loadingTitle} />
  ) : null;
  if (!notice || stagger === undefined) return notice;
  return <FadeUp index={stagger}>{notice}</FadeUp>;
}
