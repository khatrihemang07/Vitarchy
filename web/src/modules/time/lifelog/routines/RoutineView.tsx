import { useState } from "react";
import { Pencil, Trash2 } from "lucide-react";
import { type ActivityPeriod, type Routine, toTimeFailure, useDeleteRoutine, useRoutineOccurrences } from "@/api/time";
import { Button } from "@/design";
import { FailureNotice } from "../../FailureNotice";
import { ChartPanel } from "../../shared/ChartPanel";
import { IncompleteDays } from "./IncompleteDays";
import { RoutineHeadline } from "./RoutineHeadline";
import { StepDurationChart } from "./StepDurationChart";

interface RoutineViewProps {
  routine: Routine;
  period: ActivityPeriod;
  onEdit: () => void;
}

/** One saved Routine over the Period: headline times, a chart per Day, and the incomplete Days. */
export function RoutineView({ routine, period, onEdit }: RoutineViewProps) {
  const query = useRoutineOccurrences(routine.id, period);
  const remove = useDeleteRoutine();
  const [confirming, setConfirming] = useState(false);
  return (
    <ChartPanel
      title={routine.name}
      query={query}
      isEmpty={(data) => data.occurrences.length === 0}
      emptyMessage="This Routine did not happen in this Period."
      actions={
        <div className="flex flex-wrap items-center gap-2">
          <Button variant="outline" size="sm" onClick={onEdit}>
            <Pencil /> Edit
          </Button>
          {confirming ? (
            <>
              <Button variant="destructive" size="sm" disabled={remove.isPending} onClick={() => { remove.mutate(routine.id); }}>
                Delete {routine.name}?
              </Button>
              <Button variant="ghost" size="sm" onClick={() => { setConfirming(false); }}>Keep</Button>
            </>
          ) : (
            <Button variant="ghost" size="sm" onClick={() => { setConfirming(true); }}>
              <Trash2 /> Delete
            </Button>
          )}
          {remove.isError ? <FailureNotice failure={toTimeFailure(remove.error)} /> : null}
        </div>
      }
    >
      {(data) => (
        <>
          <RoutineHeadline steps={routine.steps} stats={data.stats} />
          <StepDurationChart steps={routine.steps} occurrences={data.occurrences} />
          <IncompleteDays steps={routine.steps} occurrences={data.occurrences} />
        </>
      )}
    </ChartPanel>
  );
}
