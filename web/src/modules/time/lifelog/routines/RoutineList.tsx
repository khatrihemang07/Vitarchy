import type { ActivityPeriod, Routine } from "@/api/time";
import { toTimeFailure, useRoutines } from "@/api/time";
import { FailureNotice } from "../../FailureNotice";
import { LoadingNotice } from "../../LoadingNotice";
import { RoutineView } from "./RoutineView";

interface RoutineListProps {
  period: ActivityPeriod;
  onEdit: (routine: Routine) => void;
}

/** Every saved Routine, each with its own view. Says so when there are none yet. */
export function RoutineList({ period, onEdit }: RoutineListProps) {
  const { data, isPending, isError, error } = useRoutines();
  if (isPending) return <LoadingNotice>Loading Routines…</LoadingNotice>;
  if (isError) return <FailureNotice failure={toTimeFailure(error)} />;
  if (data.length === 0) {
    return <p className="text-sm text-muted-foreground">No Routines yet. Confirm a suggestion above, or create one.</p>;
  }
  return (
    <div className="flex min-w-0 flex-col gap-6">
      {data.map((routine) => (
        <RoutineView key={routine.id} routine={routine} period={period} onEdit={() => { onEdit(routine); }} />
      ))}
    </div>
  );
}
