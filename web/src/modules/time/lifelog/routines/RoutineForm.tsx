import { useId, useState } from "react";
import { Plus } from "lucide-react";
import { type ActivityPeriod, type Routine, toTimeFailure, useLifeLogOverview, useSaveRoutine } from "@/api/time";
import { Button, DialogDescription, DialogFooter, DialogHeader, DialogTitle, Input, Label } from "@/design";
import { FailureNotice } from "../../FailureNotice";
import { emptyStep, isDraftReady, matchChoices, MAX_NAME_CHARS, MAX_STEPS, moveStep } from "./routineModel";
import { StepFields } from "./StepFields";

interface RoutineFormProps {
  /** The Routine being edited; absent for a new one. */
  routine?: Routine | undefined;
  period: ActivityPeriod;
  onDone: () => void;
}

interface Row {
  key: number;
  step: Routine["steps"][number];
}

/** The editor's body: a name and ordered Steps. Mounted fresh each time the dialog opens. */
export function RoutineForm({ routine, period, onDone }: RoutineFormProps) {
  const nameId = useId();
  const [name, setName] = useState(routine?.name ?? "");
  const [nextKey, setNextKey] = useState(() => (routine?.steps.length ?? 0) + 1);
  const [rows, setRows] = useState<Row[]>(() => (routine?.steps ?? [emptyStep()]).map((step, key) => ({ key, step })));
  const save = useSaveRoutine();
  const totals = useLifeLogOverview(period, (overview) => overview.totals).data;
  const steps = rows.map((row) => row.step);
  const choices = matchChoices(totals);

  const reorder = (index: number, delta: -1 | 1) => { setRows((current) => moveStep(current, index, delta)); };
  const submit = () => {
    save.mutate({ id: routine?.id, draft: { name, steps } }, { onSuccess: onDone });
  };
  return (
    <form
      className="flex min-w-0 flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        submit();
      }}
    >
      <DialogHeader>
        <DialogTitle>{routine ? "Edit Routine" : "New Routine"}</DialogTitle>
        <DialogDescription>Steps are matched in order within one Day, by an exact description or a project name.</DialogDescription>
      </DialogHeader>
      <div className="flex flex-col gap-1.5">
        <Label htmlFor={nameId}>Name</Label>
        <Input id={nameId} value={name} maxLength={MAX_NAME_CHARS} onChange={(event) => { setName(event.target.value); }} />
      </div>
      <ol className="flex max-h-[50vh] flex-col gap-3 overflow-y-auto pr-1" aria-label="Steps">
        {rows.map((row, index) => (
          <StepFields
            key={row.key}
            position={index}
            step={row.step}
            choices={choices}
            canMoveUp={index > 0}
            canMoveDown={index < rows.length - 1}
            onChange={(step) => { setRows((current) => current.map((r) => (r.key === row.key ? { ...r, step } : r))); }}
            onMove={(delta) => { reorder(index, delta); }}
            onRemove={() => { setRows((current) => current.filter((r) => r.key !== row.key)); }}
          />
        ))}
      </ol>
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="self-start"
        disabled={rows.length >= MAX_STEPS}
        onClick={() => {
          setRows((current) => [...current, { key: nextKey, step: emptyStep() }]);
          setNextKey((key) => key + 1);
        }}
      >
        <Plus /> Add step
      </Button>
      {save.isError ? <FailureNotice failure={toTimeFailure(save.error)} /> : null}
      <DialogFooter>
        <Button type="button" variant="ghost" onClick={onDone}>Cancel</Button>
        <Button type="submit" disabled={save.isPending || !isDraftReady(name, steps)}>
          {save.isPending ? "Saving…" : "Save"}
        </Button>
      </DialogFooter>
    </form>
  );
}
