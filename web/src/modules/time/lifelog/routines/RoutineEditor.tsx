import type { ActivityPeriod, Routine } from "@/api/time";
import { Dialog, DialogContent } from "@/design";
import { RoutineForm } from "./RoutineForm";

interface RoutineEditorProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** The Routine to edit; absent to create one. */
  routine?: Routine | undefined;
  period: ActivityPeriod;
}

/** A dialog to create or edit a Routine. Focus is trapped and Escape closes it. */
export function RoutineEditor({ open, onOpenChange, routine, period }: RoutineEditorProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
        <RoutineForm routine={routine} period={period} onDone={() => { onOpenChange(false); }} />
      </DialogContent>
    </Dialog>
  );
}
