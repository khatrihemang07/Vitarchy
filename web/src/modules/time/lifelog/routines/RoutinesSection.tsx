import { useState } from "react";
import { Plus } from "lucide-react";
import type { ActivityPeriod, Routine } from "@/api/time";
import { Button } from "@/design";
import { RoutineEditor } from "./RoutineEditor";
import { RoutineList } from "./RoutineList";
import { RoutineSuggestions } from "./RoutineSuggestions";

/** Routines in the Life log tab: suggestions to confirm, the saved Routines, and the editor. */
export function RoutinesSection({ period }: { period: ActivityPeriod }) {
  // `undefined` is closed, `null` is a new Routine, a Routine is an edit.
  const [editing, setEditing] = useState<Routine | null | undefined>(undefined);
  return (
    <section aria-labelledby="routines-heading" className="flex min-w-0 flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h2 id="routines-heading" className="text-lg font-semibold">Routines</h2>
        <Button variant="outline" size="sm" onClick={() => { setEditing(null); }}>
          <Plus /> New Routine
        </Button>
      </div>
      <RoutineSuggestions period={period} />
      <RoutineList period={period} onEdit={setEditing} />
      <RoutineEditor
        open={editing !== undefined}
        onOpenChange={(open) => { if (!open) setEditing(undefined); }}
        routine={editing ?? undefined}
        period={period}
      />
    </section>
  );
}
