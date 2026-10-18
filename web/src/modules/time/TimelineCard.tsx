import { useRebuildTimeline, useTimeStatus, type TimelineStatus } from "@/api/time";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, MotionButton } from "@/design";

import { errorMessage } from "./failureMessage";
import { formatCount, formatDay, formatSpan } from "./format";
import { timelineFigures } from "./timelineFigures";
import { TimeNotice } from "./TimeNotice";

function withShare(secs: number, share: number | null): string {
  return share === null ? formatSpan(secs) : `${formatSpan(secs)} (${String(share)}% of covered time)`;
}

function TimelineFigureList({ timeline }: { timeline: TimelineStatus }) {
  const { awayShare, unknownShare } = timelineFigures(timeline);
  const rows: [string, string][] = [
    ["Segments", formatCount(timeline.segment_count)],
    ["Covers", `${formatDay(timeline.oldest ?? "")} to ${formatDay(timeline.newest ?? "")}`],
    ["Away", withShare(timeline.away_secs, awayShare)],
    ["Away unknown", withShare(timeline.unknown_secs, unknownShare)],
  ];
  return (
    <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
      {rows.map(([label, value]) => (
        <div key={label} className="contents">
          <dt className="text-muted-foreground">{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

/** The stored Timeline's figures, with a button to rebuild it from Activity. */
export function TimelineCard() {
  const status = useTimeStatus();
  const rebuild = useRebuildTimeline();
  // Loading and failed-to-load are shown once by the Import cards beside this one.
  if (!status.data) return null;
  const timeline = status.data.timeline;

  return (
    <Card>
      <CardHeader>
        <CardTitle>Timeline</CardTitle>
        <CardDescription>
          One account of what happened on this computer, built from every Source&apos;s Activity. Rebuilding is safe: it is always
          derived from Activity.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-1 flex-col gap-4">
        {timeline.segment_count === 0 ? (
          <p className="text-sm text-muted-foreground">No Segments yet. Import Activity from a Source to build the Timeline.</p>
        ) : (
          <>
            <TimelineFigureList timeline={timeline} />
            <p className="text-sm text-muted-foreground">
              Away is unknown where no Source could tell whether you were away.
            </p>
          </>
        )}
        {rebuild.isError ? <TimeNotice inline kind="failure" title={errorMessage(rebuild.error)} /> : null}
        {rebuild.isSuccess ? (
          <p role="status" className="text-sm">
            Rebuilt: {formatCount(rebuild.data.segment_count)} Segments from {formatCount(rebuild.data.activity_count)} Activity
            rows.
          </p>
        ) : null}
        <MotionButton
          className="mt-auto self-start"
          disabled={rebuild.isPending}
          onClick={() => {
            rebuild.mutate();
          }}
        >
          {rebuild.isPending ? "Rebuilding…" : "Rebuild Timeline"}
        </MotionButton>
      </CardContent>
    </Card>
  );
}
