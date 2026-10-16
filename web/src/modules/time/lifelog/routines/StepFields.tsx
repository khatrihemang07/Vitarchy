import { useId } from "react";
import { ChevronDown, ChevronUp, Trash2 } from "lucide-react";
import type { RoutineMatchKind, RoutineStep } from "@/api/time";
import { Button, Input, Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/design";
import { MAX_LABEL_CHARS, MAX_VALUE_CHARS } from "./routineLimits";
import { type MatchChoices, withCurrent } from "./routineModel";

interface StepFieldsProps {
  position: number;
  step: RoutineStep;
  choices: MatchChoices;
  canMoveUp: boolean;
  canMoveDown: boolean;
  onChange: (step: RoutineStep) => void;
  onMove: (delta: -1 | 1) => void;
  onRemove: () => void;
}

const KINDS: readonly { value: RoutineMatchKind; label: string }[] = [
  { value: "description", label: "Description" },
  { value: "project", label: "Project" },
];

/** One Step in the editor: its label, what it matches, and controls to reorder or remove it. */
export function StepFields({ position, step, choices, canMoveUp, canMoveDown, onChange, onMove, onRemove }: StepFieldsProps) {
  const id = useId();
  const known = choices[step.kind];
  const number = String(position + 1);
  return (
    <li className="flex min-w-0 flex-col gap-2 rounded-lg border bg-background/40 p-3">
      <div className="flex items-center justify-between gap-2">
        <span className="text-sm font-medium">Step {number}</span>
        <div className="flex gap-1">
          <Button type="button" variant="ghost" size="icon-sm" aria-label={`Move step ${number} up`} disabled={!canMoveUp} onClick={() => { onMove(-1); }}>
            <ChevronUp />
          </Button>
          <Button type="button" variant="ghost" size="icon-sm" aria-label={`Move step ${number} down`} disabled={!canMoveDown} onClick={() => { onMove(1); }}>
            <ChevronDown />
          </Button>
          <Button type="button" variant="ghost" size="icon-sm" aria-label={`Remove step ${number}`} onClick={onRemove}>
            <Trash2 />
          </Button>
        </div>
      </div>
      <div className="grid gap-2 sm:grid-cols-3">
        <div className="flex flex-col gap-1.5">
          <Label htmlFor={`${id}-label`}>Label</Label>
          <Input id={`${id}-label`} value={step.label} maxLength={MAX_LABEL_CHARS} onChange={(event) => { onChange({ ...step, label: event.target.value }); }} />
        </div>
        <div className="flex flex-col gap-1.5">
          <Label htmlFor={`${id}-kind`}>Matches a</Label>
          <Select value={step.kind} onValueChange={(kind) => { onChange({ ...step, kind: kind as RoutineMatchKind, value: "" }); }}>
            <SelectTrigger id={`${id}-kind`} className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {KINDS.map((kind) => (
                <SelectItem key={kind.value} value={kind.value}>{kind.label}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex flex-col gap-1.5">
          <Label htmlFor={`${id}-value`}>{step.kind === "project" ? "Project" : "Description"}</Label>
          {known.length > 0 ? (
            <Select value={step.value} onValueChange={(value) => { onChange({ ...step, value, label: step.label.trim() === "" ? value : step.label }); }}>
              <SelectTrigger id={`${id}-value`} className="w-full">
                <SelectValue placeholder="Choose…" />
              </SelectTrigger>
              <SelectContent>
                {withCurrent(known, step.value).map((option) => (
                  <SelectItem key={option} value={option}>{option}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : (
            <Input id={`${id}-value`} value={step.value} maxLength={MAX_VALUE_CHARS} placeholder="Type it exactly" onChange={(event) => { onChange({ ...step, value: event.target.value }); }} />
          )}
        </div>
      </div>
    </li>
  );
}
