import { errorMessage } from "./failureMessage";
import { TimeNotice } from "./TimeNotice";

interface StatusNoticesProps {
  /** The Time status query: only the parts the notices need. */
  status: { isPending: boolean; isError: boolean; error: unknown; refetch: () => Promise<unknown> };
  loadingTitle: string;
  /** Sits inside a card (no surface of its own); a full-width notice otherwise. */
  inline?: boolean;
}

/** The loading and failed-to-load notices of the Time status, so each card handles the two states the same way. */
export function StatusNotices({ status, loadingTitle, inline = false }: StatusNoticesProps) {
  if (status.isPending) return <TimeNotice inline={inline} kind="loading" title={loadingTitle} />;
  if (status.isError) {
    return <TimeNotice inline={inline} kind="failure" title={errorMessage(status.error)} onRetry={() => void status.refetch()} />;
  }
  return null;
}
