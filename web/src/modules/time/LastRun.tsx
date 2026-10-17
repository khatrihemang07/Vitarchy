import type { TimeRun } from "@/api/time";
import { describeRun } from "./describeRun";
import { formatAge } from "./format";

interface LastRunProps {
  /** "Sync" or "import": what the run was. */
  noun: string;
  run: TimeRun;
  now: Date;
}

/** "Last sync 2h ago: 5 new …": how the latest run of a Sync or Import went. */
export function LastRun({ noun, run, now }: LastRunProps) {
  return (
    <p className="text-sm" data-outcome={run.outcome}>
      <span className="text-muted-foreground">
        Last {noun} {formatAge(run.finished_at, now)}:{" "}
      </span>
      {describeRun(run)}
    </p>
  );
}
