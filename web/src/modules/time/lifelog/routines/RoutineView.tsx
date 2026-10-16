import { useState } from "react";
import { Pencil, Trash2 } from "lucide-react";
import { type RoutineOccurrences, toTimeFailure, useDeleteRoutine } from "@/api/time";
import { Button } from "@/design";
import { FailureNotice } from "../../FailureNotice";
import { ChartPanel } from "../../shared/ChartPanel";
import { IncompleteDays } from "./IncompleteDays";
import { RoutineHeadline } from "./RoutineHeadline";
import { StepDurationChart } from "./StepDurationChart";

interface RoutineViewProps {
  /** The Routine with its Occurrences and statistics; its Steps are the only source of labels. */
  entry: RoutineOccurrences;
  onEdit: () => void;
}

/** One saved Routine over the Period: headline times, a chart per Day, and the incomplete Days. */
export function RoutineView({ entry, onEdit }: RoutineViewProps) {
  const { routine } = entry;
  const remove = useDeleteRoutine();
  const [confirming, setConfirming] = useState(false);
  return (
    <ChartPanel
      title={routine.name}
      query={{ isPending: false, isError: false, error: null, data: entry }}
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
