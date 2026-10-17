import { toTimeFailure, useImportActivity, type SourceActivityStatus } from "@/api/time";
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/design";

import { describeRun } from "./describeRun";
import { FailureNotice } from "./FailureNotice";
import { formatAge, formatCount, formatDay } from "./format";
import { sourceCopy } from "./sourceCopy";
import { useClock } from "./useClock";

function storedSpan(activity: SourceActivityStatus): string {
  if (activity.activity_count === 0 || !activity.oldest || !activity.newest) return "No Activity imported yet.";
  const apps = `${formatCount(activity.app_count)} apps`;
  const span = `${formatDay(activity.oldest)} to ${formatDay(activity.newest)}`;
  return `${formatCount(activity.activity_count)} Activity rows from ${apps}, ${span}.`;
}

/** Start an Import of one Source from its latest Snapshot and see how the last one went. */
export function ActivityImportCard({ activity }: { activity: SourceActivityStatus }) {
  const importActivity = useImportActivity(activity.source);
  const now = useClock();
  const copy = sourceCopy[activity.source];
  const lastImport = activity.last_import;
  const hasSnapshot = Boolean(activity.snapshot_taken_at);

  return (
    <Card>
      <CardHeader>
        <CardTitle>{copy.title}</CardTitle>
        <CardDescription>{copy.description}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <p className="text-sm text-muted-foreground">{storedSpan(activity)}</p>
        {activity.snapshot_taken_at ? (
          <p className="text-sm text-muted-foreground">Snapshot taken {formatAge(activity.snapshot_taken_at, now)}.</p>
        ) : (
          <p className="text-sm text-muted-foreground">
            No Snapshot yet — install the helper: <code>make helper-install</code>
          </p>
        )}
        {lastImport ? (
          <p className="text-sm" data-outcome={lastImport.outcome}>
            <span className="text-muted-foreground">Last import {formatAge(lastImport.finished_at, now)}: </span>
            {describeRun(lastImport)}
          </p>
        ) : (
          <p className="text-sm text-muted-foreground">Not imported yet.</p>
        )}
        {importActivity.isError ? <FailureNotice failure={toTimeFailure(importActivity.error)} /> : null}
        <Button
          className="self-start"
          disabled={!hasSnapshot || importActivity.isPending}
          onClick={() => {
            importActivity.mutate();
          }}
        >
          {importActivity.isPending ? "Importing…" : copy.title}
        </Button>
      </CardContent>
    </Card>
  );
}
