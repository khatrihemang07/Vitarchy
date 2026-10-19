import type { TransferCandidate } from "@/api/finance";
import { Checkbox, Label, RadioGroup, RadioGroupItem } from "@/design";
import { formatDate } from "../dates";
import { formatInr } from "../money";

interface CandidateListProps {
  candidates: readonly TransferCandidate[];
  names: ReadonlyMap<number, string>;
  /** Whether several may be chosen (a Money-out row can be paid into several Money-in rows). */
  several: boolean;
  selected: readonly number[];
  onToggle: (id: number) => void;
}

function Line({ candidate, names }: { candidate: TransferCandidate; names: ReadonlyMap<number, string> }) {
  return (
    <span className="flex min-w-0 flex-1 flex-col">
      <span className="[overflow-wrap:anywhere]">{candidate.description}</span>
      <span className="text-xs text-muted-foreground">
        {formatDate(candidate.date)} · {names.get(candidate.account_id) ?? "Account"} · {formatInr(candidate.amount_paise)}
      </span>
    </span>
  );
}

/** The Transactions a row could be linked to, chosen with checkboxes (several) or radios (one). */
export function CandidateList({ candidates, names, several, selected, onToggle }: CandidateListProps) {
  if (!several) {
    return (
      <RadioGroup value={selected[0]?.toString() ?? ""} onValueChange={(value) => { onToggle(Number(value)); }} aria-label="Money-out Transactions">
        {candidates.map((c) => (
          <Label key={c.id} className="items-start gap-3 rounded-md border p-3 font-normal">
            <RadioGroupItem value={c.id.toString()} className="mt-0.5" />
            <Line candidate={c} names={names} />
          </Label>
        ))}
      </RadioGroup>
    );
  }
  return (
    <ul aria-label="Money-in Transactions" className="flex flex-col gap-2">
      {candidates.map((c) => (
        <li key={c.id}>
          <Label className="items-start gap-3 rounded-md border p-3 font-normal">
            <Checkbox className="mt-0.5" checked={selected.includes(c.id)} onCheckedChange={() => { onToggle(c.id); }} />
            <Line candidate={c} names={names} />
          </Label>
        </li>
      ))}
    </ul>
  );
}
