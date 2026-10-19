import { useSyncTime, useTimeStatus, type TimeStatus } from "@/api/time";
import { Card, MotionButton, CardContent, CardDescription, CardHeader, CardTitle, Notice } from "@/design";
import { errorMessage } from "./failureMessage";
import { formatCount, formatDay } from "./format";
import { LastRun } from "./LastRun";
import { useClock } from "./useClock";

function mirroredSpan(status: TimeStatus): string {
  if (status.time_entry_count === 0 || !status.oldest_entry || !status.newest_entry) {
    return "No Time Entries yet.";
  }
  const from = formatDay(status.oldest_entry);
  const to = formatDay(status.newest_entry);
  return `${formatCount(status.time_entry_count)} Time Entries, ${from} to ${to}.`;
}

/** Start a Sync and see how the last one went. */
export function SyncCard() {
  const status = useTimeStatus();
  const sync = useSyncTime();
  const now = useClock();
  const connected = status.data?.connected === true;
  const lastSync = status.data?.last_sync;
  const startSync = () => {
    sync.mutate();
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Sync from Toggl</CardTitle>
        <CardDescription>
          Brings your Time Entries from the last 3 months into Vitarchy and keeps the copy matching Toggl.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-1 flex-col gap-4">
        {status.data ? <p className="text-sm text-muted-foreground">{mirroredSpan(status.data)}</p> : null}
        {lastSync ? <LastRun noun="sync" run={lastSync} now={now} /> : null}
        {status.data && !lastSync ? <p className="text-sm text-muted-foreground">Not synced yet.</p> : null}
        {sync.isError ? <Notice inline kind="failure" title={errorMessage(sync.error)} /> : null}
        {status.data && !connected ? (
          <p className="text-sm text-muted-foreground">Connect Toggl to start syncing.</p>
        ) : null}
        <MotionButton className="mt-auto self-start" disabled={!connected || sync.isPending} onClick={startSync}>
          {sync.isPending ? "Syncing…" : "Sync now"}
        </MotionButton>
      </CardContent>
    </Card>
  );
}
