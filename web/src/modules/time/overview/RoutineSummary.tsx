import { toTimeFailure, useRoutineOccurrences, type ActivityPeriod } from "@/api/time";
import { FailureNotice } from "../FailureNotice";
import { formatSpan } from "../shared/duration";

/** Each saved Routine's usual door-to-door time and how many complete Days it rests on. */
export function RoutineSummary({ period }: { period: ActivityPeriod }) {
  const query = useRoutineOccurrences(period);
  if (query.isPending) return <p role="status" className="text-sm text-muted-foreground">Loading Routines…</p>;
  if (query.isError) return <FailureNotice failure={toTimeFailure(query.error)} />;
  if (query.data.length === 0) {
    return <p className="text-sm text-muted-foreground">Save a Routine in the Life log tab to see its door-to-door time here.</p>;
  }
  return (
    <ul aria-label="Routines" className="flex flex-col gap-2">
      {query.data.map(({ routine, stats }) => (
        <li key={routine.id} className="flex flex-wrap items-baseline justify-between gap-x-3 text-sm">
          <span className="min-w-0 truncate font-medium" title={routine.name}>{routine.name}</span>
          <span className="tabular-nums">
            {stats.door_to_door ? (
              <>
                <span className="font-medium">{formatSpan(stats.door_to_door.median_seconds)}</span>{" "}
                <span className="text-xs text-muted-foreground">door to door, {String(stats.complete_days)} complete {stats.complete_days === 1 ? "day" : "days"}</span>
              </>
            ) : (
              <span className="text-xs text-muted-foreground">No complete day yet</span>
            )}
          </span>
        </li>
      ))}
    </ul>
  );
}
