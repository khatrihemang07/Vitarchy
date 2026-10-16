import { toTimeFailure, useImportActivity, useTimeStatus, type ActivityStatus } from "@/api/time";
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/design";

import { describeRun } from "./describeRun";
import { FailureNotice } from "./FailureNotice";
import { formatAge, formatCount, formatDay } from "./format";
import { LoadingNotice } from "./LoadingNotice";
import { useClock } from "./useClock";

function storedSpan(activity: ActivityStatus): string {
  if (activity.activity_count === 0 || !activity.oldest || !activity.newest) return "No Activity imported yet.";
  const apps = `${formatCount(activity.app_count)} apps`;
  const span = `${formatDay(activity.oldest)} to ${formatDay(activity.newest)}`;
  return `${formatCount(activity.activity_count)} Activity rows from ${apps}, ${span}.`;
}

/** Start an Import from the latest Snapshot and see how the last one went. */
export function ActivityImportCard() {
  const status = useTimeStatus();
  const importActivity = useImportActivity();
  const now = useClock();
  const activity = status.data?.activity;
  const lastImport = activity?.last_import;
  const hasSnapshot = Boolean(activity?.snapshot_taken_at);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Import activity</CardTitle>
        <CardDescription>
          What the Toggl Track app noticed on this computer, kept as Activity. Importing only adds, so
          Vitarchy keeps what Toggl later discards.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {status.isPending ? <LoadingNotice>Loading status…</LoadingNotice> : null}
        {status.isError ? <FailureNotice failure={toTimeFailure(status.error)} /> : null}
        {activity ? <p className="text-sm text-muted-foreground">{storedSpan(activity)}</p> : null}
        {activity?.snapshot_taken_at ? (
          <p className="text-sm text-muted-foreground">
            Snapshot taken {formatAge(activity.snapshot_taken_at, now)}.
          </p>
        ) : null}
        {activity && !hasSnapshot ? (
          <p className="text-sm text-muted-foreground">
            No Snapshot yet — install the helper: <code>make helper-install</code>
          </p>
        ) : null}
        {lastImport ? (
          <p className="text-sm" data-outcome={lastImport.outcome}>
            <span className="text-muted-foreground">Last import {formatAge(lastImport.finished_at, now)}: </span>
            {describeRun(lastImport)}
          </p>
        ) : null}
        {activity && !lastImport ? <p className="text-sm text-muted-foreground">Not imported yet.</p> : null}
        {importActivity.isError ? <FailureNotice failure={toTimeFailure(importActivity.error)} /> : null}
        <Button
          className="self-start"
          disabled={!hasSnapshot || importActivity.isPending}
          onClick={() => {
            importActivity.mutate();
          }}
        >
          {importActivity.isPending ? "Importing…" : "Import activity"}
        </Button>
      </CardContent>
    </Card>
  );
}
