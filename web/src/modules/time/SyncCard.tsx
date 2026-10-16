import { toTimeFailure, useSyncTime, useTimeStatus, type TimeStatus } from "@/api/time";
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/design";
import { describeRun } from "./describeRun";
import { FailureNotice } from "./FailureNotice";
import { formatAge, formatCount, formatDay } from "./format";
import { LoadingNotice } from "./LoadingNotice";
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

  return (
    <Card>
      <CardHeader>
        <CardTitle>Sync from Toggl</CardTitle>
        <CardDescription>
          Brings your Time Entries from the last 3 months into Vitarchy and keeps the copy matching Toggl.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {status.isPending ? <LoadingNotice>Loading status…</LoadingNotice> : null}
        {status.isError ? <FailureNotice failure={toTimeFailure(status.error)} /> : null}
        {status.data ? <p className="text-sm text-muted-foreground">{mirroredSpan(status.data)}</p> : null}
        {lastSync ? (
          <p className="text-sm" data-outcome={lastSync.outcome}>
            <span className="text-muted-foreground">Last sync {formatAge(lastSync.finished_at, now)}: </span>
            {describeRun(lastSync)}
          </p>
        ) : null}
        {status.data && !lastSync ? <p className="text-sm text-muted-foreground">Not synced yet.</p> : null}
        {sync.isError ? <FailureNotice failure={toTimeFailure(sync.error)} /> : null}
        {status.data && !connected ? (
          <p className="text-sm text-muted-foreground">Connect Toggl to start syncing.</p>
        ) : null}
        <Button className="self-start" disabled={!connected || sync.isPending} onClick={() => {
            sync.mutate();
          }}>
          {sync.isPending ? "Syncing…" : "Sync now"}
        </Button>
      </CardContent>
    </Card>
  );
}
