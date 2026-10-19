import { useFinanceStatus } from "@/api/finance";
import { Notice, QueryNotices, Skeleton } from "@/design";
import { errorMessage } from "../failureMessage";

/**
 * One line tall like the ready message below. The Import page only renders ReaderStatus once the
 * status has answered, so the taller missing-reader warning appears with the page, not after it.
 */
const readerSkeleton = (
  <div role="status" aria-busy="true" className="flex min-h-6 items-center">
    <span className="sr-only">Checking the statement reader…</span>
    <Skeleton className="h-4 w-56" />
  </div>
);

/** Whether the statement reader can be found; when it cannot, how to install it. */
export function ReaderStatus() {
  const status = useFinanceStatus();
  if (!status.data) {
    return <QueryNotices query={status} describeError={errorMessage} skeleton={readerSkeleton} />;
  }
  if (!status.data.reader_available) {
    return (
      <Notice kind="warning" title="The statement reader is not installed">
        Install the reader: <code>make finance-reader-install</code>
      </Notice>
    );
  }
  return (
    <p className="flex min-h-6 items-center gap-2 text-sm text-muted-foreground" role="status">
      <span aria-hidden="true" className="size-2 rounded-full bg-success" />
      The statement reader is ready.
    </p>
  );
}
