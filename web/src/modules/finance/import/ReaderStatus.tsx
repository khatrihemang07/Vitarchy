import { useFinanceStatus } from "@/api/finance";
import { QueryNotices } from "../QueryNotices";
import { FinanceNotice } from "../FinanceNotice";

/** Whether the statement reader can be found; when it cannot, how to install it. */
export function ReaderStatus() {
  const status = useFinanceStatus();
  if (status.isPending || status.isError) return <QueryNotices query={status} loadingTitle="Checking the statement reader…" />;
  if (!status.data.reader_available) {
    return (
      <FinanceNotice kind="warning" title="The statement reader is not installed">
        Install the reader: <code>make finance-reader-install</code>
      </FinanceNotice>
    );
  }
  return (
    <p className="flex items-center gap-2 text-sm text-muted-foreground" role="status">
      <span aria-hidden="true" className="size-2 rounded-full bg-success" />
      The statement reader is ready.
    </p>
  );
}
