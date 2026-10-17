import { ChevronLeft, ChevronRight } from "lucide-react";
import type { PeriodKind } from "@/api/time";
import { Button, Input, SegmentedControl, type SegmentedOption } from "@/design";
import { type DayRange, periodLabel } from "./period";

const OPTIONS: readonly SegmentedOption[] = [
  { value: "day", label: "Day" },
  { value: "week", label: "Week" },
  { value: "month", label: "Month" },
  { value: "custom", label: "Custom" },
];

const NOUNS: Record<PeriodKind, string> = { day: "Day", week: "week", month: "month", custom: "stretch" };

interface PeriodPickerProps {
  kind: PeriodKind;
  range: DayRange;
  /** True when the shown Period already holds today, so Today has nothing to do. */
  isCurrent: boolean;
  onKind: (kind: PeriodKind) => void;
  onShift: (direction: 1 | -1) => void;
  onToday: () => void;
  onCustom: (range: DayRange) => void;
}

const asKind = (value: string): PeriodKind => OPTIONS.find((o) => o.value === value)?.value as PeriodKind | undefined ?? "week";

/** Chooses the Period: its kind, which one (arrows, Today) and, for Custom, both end Days. */
export function PeriodPicker({ kind, range, isCurrent, onKind, onShift, onToday, onCustom }: PeriodPickerProps) {
  const noun = NOUNS[kind];
  return (
    <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
      <SegmentedControl label="Period" value={kind} onValueChange={(v) => { onKind(asKind(v)); }} options={OPTIONS} />
      <div className="flex items-center gap-1">
        <Button variant="outline" size="icon-sm" aria-label={`Previous ${noun}`} onClick={() => { onShift(-1); }}>
          <ChevronLeft aria-hidden="true" />
        </Button>
        <span aria-live="polite" className="min-w-36 px-2 text-center text-sm font-medium tabular-nums">
          {periodLabel(kind, range)}
        </span>
        <Button variant="outline" size="icon-sm" aria-label={`Next ${noun}`} onClick={() => { onShift(1); }}>
          <ChevronRight aria-hidden="true" />
        </Button>
      </div>
      <Button variant="outline" size="sm" disabled={isCurrent} onClick={onToday}>
        Today
      </Button>
      {kind === "custom" ? (
        <div className="flex flex-wrap items-center gap-2 text-sm">
          <label className="flex items-center gap-1.5 text-muted-foreground">
            From
            <Input type="date" className="h-8 w-auto text-sm" value={range.from} max={range.to} onChange={(e) => { if (e.target.value) onCustom({ from: e.target.value, to: range.to }); }} />
          </label>
          <label className="flex items-center gap-1.5 text-muted-foreground">
            To
            <Input type="date" className="h-8 w-auto text-sm" value={range.to} min={range.from} onChange={(e) => { if (e.target.value) onCustom({ from: range.from, to: e.target.value }); }} />
          </label>
        </div>
      ) : null}
    </div>
  );
}
