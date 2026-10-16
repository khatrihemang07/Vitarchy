import { Input, Label } from "@/design";
import { isIsoDate, type PeriodSelection } from "./period";

interface CustomRangeProps {
  value: PeriodSelection;
  /** Latest date that can be picked. */
  max: string;
  onChange: (next: PeriodSelection) => void;
}

/** From and to dates of a custom Period. An unfinished or reversed entry is ignored. */
export function CustomRange({ value, max, onChange }: CustomRangeProps) {
  const set = (edit: Partial<Pick<PeriodSelection, "from" | "to">>) => {
    const next = { ...value, ...edit };
    if (isIsoDate(next.from) && isIsoDate(next.to) && next.from <= next.to) onChange(next);
  };
  return (
    <div className="flex flex-wrap items-center gap-3">
      <Label className="flex items-center gap-2 text-sm">
        From
        <Input type="date" value={value.from} max={max} className="w-40" onChange={(e) => { set({ from: e.target.value }); }} />
      </Label>
      <Label className="flex items-center gap-2 text-sm">
        To
        <Input type="date" value={value.to} max={max} className="w-40" onChange={(e) => { set({ to: e.target.value }); }} />
      </Label>
    </div>
  );
}
