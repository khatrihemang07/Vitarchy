import { useState } from "react";
import { Input, Label } from "@/design";
import { isIsoDate, MAX_PERIOD_DAYS, spanDays, type PeriodSelection } from "./period";

interface CustomRangeProps {
  value: PeriodSelection;
  /** Latest date that can be picked. */
  max: string;
  onChange: (next: PeriodSelection) => void;
}

type Field = "from" | "to";

/** Why `from..to` cannot be shown, or null when it can. */
function problemWith(from: string, to: string, max: string): string | null {
  if (to > max || from > max) return "Pick today or an earlier date; the future has no data yet.";
  if (spanDays(from, to) > MAX_PERIOD_DAYS) return `A range can cover at most ${String(MAX_PERIOD_DAYS)} days.`;
  return null;
}

/**
 * From and to dates of a custom Period. Edits live in a draft so a half-typed or cleared field
 * stays as typed; the Period changes only once both dates are valid. Setting one end past the
 * other moves the other end with it.
 */
export function CustomRange({ value, max, onChange }: CustomRangeProps) {
  const [draft, setDraft] = useState({ from: value.from, to: value.to });
  const [problem, setProblem] = useState<string | null>(null);
  // A range chosen elsewhere (Previous, Next) replaces the draft; edits made here are echoed back unchanged.
  const [seen, setSeen] = useState({ from: value.from, to: value.to });
  if (seen.from !== value.from || seen.to !== value.to) {
    setSeen({ from: value.from, to: value.to });
    setDraft({ from: value.from, to: value.to });
    setProblem(null);
  }

  const edit = (field: Field, text: string) => {
    const next = { ...draft, [field]: text };
    if (!isIsoDate(next.from) || !isIsoDate(next.to)) {
      setDraft(next);
      setProblem(null);
      return;
    }
    if (next.from > next.to) next[field === "from" ? "to" : "from"] = text;
    setDraft(next);
    const why = problemWith(next.from, next.to, max);
    setProblem(why);
    if (!why) onChange({ ...value, from: next.from, to: next.to });
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-3">
        <Label className="flex items-center gap-2 text-sm">
          From
          <Input type="date" value={draft.from} max={max} className="w-40" onChange={(e) => { edit("from", e.target.value); }} />
        </Label>
        <Label className="flex items-center gap-2 text-sm">
          To
          <Input type="date" value={draft.to} max={max} className="w-40" onChange={(e) => { edit("to", e.target.value); }} />
        </Label>
      </div>
      {problem ? <p role="alert" className="text-sm text-destructive">{problem}</p> : null}
    </div>
  );
}
