import { errorMessage } from "./failureMessage";
import { FinanceNotice } from "./FinanceNotice";

interface QueryNoticesProps {
  /** A TanStack query: only the parts the notices need. */
  query: { isPending: boolean; isError: boolean; error: unknown; refetch: () => Promise<unknown> };
  loadingTitle: string;
  /** Sits inside a card (no surface of its own); a full-width notice otherwise. */
  inline?: boolean;
}

/** The loading and failed-to-load notices of a query, so every list handles the two states the same way. */
export function QueryNotices({ query, loadingTitle, inline = false }: QueryNoticesProps) {
  if (query.isPending) return <FinanceNotice inline={inline} kind="loading" title={loadingTitle} />;
  if (query.isError) {
    return <FinanceNotice inline={inline} kind="failure" title={errorMessage(query.error)} onRetry={() => void query.refetch()} />;
  }
  return null;
}
