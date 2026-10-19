import { useFinanceStatus } from "@/api/finance";
import { FadeUp, Notice, QueryNotices } from "@/design";
import { errorMessage } from "../failureMessage";

/** Whether the statement reader can be found; when it cannot, how to install it. Brings its own entrance step, so nothing sits in the page while it loads. */
export function ReaderStatus({ stagger }: { stagger: number }) {
  const status = useFinanceStatus();
  if (!status.data) {
    return <QueryNotices query={status} describeError={errorMessage} loadingTitle="Checking the statement reader…" stagger={stagger} />;
  }
  return (
    <FadeUp index={stagger}>
      {status.data.reader_available ? (
        <p className="flex items-center gap-2 text-sm text-muted-foreground" role="status">
          <span aria-hidden="true" className="size-2 rounded-full bg-success" />
          The statement reader is ready.
        </p>
      ) : (
        <Notice kind="warning" title="The statement reader is not installed">
          Install the reader: <code>make finance-reader-install</code>
        </Notice>
      )}
    </FadeUp>
  );
}
