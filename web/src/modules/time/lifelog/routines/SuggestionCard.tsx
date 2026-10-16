import { useId, useState } from "react";
import { type RoutineSuggestion, toTimeFailure, useSaveRoutine } from "@/api/time";
import { Button, Input, Label } from "@/design";
import { FailureNotice } from "../../FailureNotice";
import { MAX_NAME_CHARS, stepsOfSuggestion, suggestedName } from "./routineModel";
import { StepChain } from "./StepChain";

interface SuggestionCardProps {
  suggestion: RoutineSuggestion;
  onDismiss: () => void;
}

/** One suggested Routine: its Steps, how often it happened, a name to confirm or change. */
export function SuggestionCard({ suggestion, onDismiss }: SuggestionCardProps) {
  const nameId = useId();
  const [name, setName] = useState(() => suggestedName(suggestion));
  const save = useSaveRoutine();
  const confirm = () => {
    save.mutate({ draft: { name, steps: stepsOfSuggestion(suggestion) } });
  };
  return (
    <li className="flex min-w-0 flex-col gap-3 rounded-lg border bg-background/40 p-3">
      <StepChain labels={suggestion.steps} />
      <p className="text-xs text-muted-foreground">Happened in this order on {String(suggestion.days)} days.</p>
      <div className="flex flex-col gap-1.5">
        <Label htmlFor={nameId}>Name</Label>
        <Input id={nameId} value={name} maxLength={MAX_NAME_CHARS} onChange={(event) => { setName(event.target.value); }} aria-invalid={save.isError} />
      </div>
      {save.isError ? <FailureNotice failure={toTimeFailure(save.error)} /> : null}
      <div className="flex gap-2">
        <Button size="sm" onClick={confirm} disabled={save.isPending || name.trim() === ""}>
          {save.isPending ? "Saving…" : "Confirm"}
        </Button>
        <Button size="sm" variant="ghost" onClick={onDismiss} disabled={save.isPending}>
          Dismiss
        </Button>
      </div>
    </li>
  );
}
