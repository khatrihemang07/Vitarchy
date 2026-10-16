import type { RoutineOccurrence, RoutineStep } from "@/api/time";
import { formatDayLong } from "../../shared/dayLabel";
import { incompleteDays } from "./routineModel";

/** The Days a Step was missing, and which Steps. Renders nothing when every Day was complete. */
export function IncompleteDays({ steps, occurrences }: { steps: readonly RoutineStep[]; occurrences: readonly RoutineOccurrence[] }) {
  const days = incompleteDays(occurrences, steps);
  if (days.length === 0) return null;
  return (
    <div className="flex flex-col gap-1">
      <h4 className="text-sm font-medium">Incomplete days</h4>
      <p className="text-xs text-muted-foreground">These count for the Steps that happened, but not for the door-to-door time.</p>
      <div className="max-h-56 overflow-auto">
        <table className="w-full text-left text-xs">
          <caption className="sr-only">Days on which a Step of this Routine was missing</caption>
          <thead>
            <tr>
              <th scope="col" className="sticky top-0 bg-card px-2 py-1 font-medium text-muted-foreground">Day</th>
              <th scope="col" className="sticky top-0 bg-card px-2 py-1 font-medium text-muted-foreground">Missing</th>
            </tr>
          </thead>
          <tbody>
            {days.map((day) => (
              <tr key={day.date} className="border-t">
                <td className="px-2 py-1 whitespace-nowrap">{formatDayLong(day.date)}</td>
                <td className="px-2 py-1">{day.missing.join(", ")}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
