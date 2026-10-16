import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button, SegmentedControl } from "@/design";
import { CustomRange } from "./CustomRange";
import {
  canGoNext,
  labelPeriod,
  PERIOD_KINDS,
  type PeriodKind,
  type PeriodSelection,
  shiftPeriod,
  switchKind,
  type WeekStart,
} from "./period";

interface PeriodPickerProps {
  value: PeriodSelection;
  weekStart: WeekStart;
  /** Today's local date; the picker never moves past it. */
  today: string;
  onChange: (next: PeriodSelection) => void;
}

/** Choose the Period to look at: a day, week, month or custom range, with previous and next. */
export function PeriodPicker({ value, weekStart, today, onChange }: PeriodPickerProps) {
  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-3">
        <SegmentedControl
          label="Period"
          value={value.kind}
          options={PERIOD_KINDS}
          onValueChange={(kind) => { onChange(switchKind(value, kind as PeriodKind, weekStart)); }}
        />
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="icon" aria-label="Previous period" onClick={() => { onChange(shiftPeriod(value, -1)); }}>
            <ChevronLeft />
          </Button>
          <span className="min-w-40 text-center text-sm font-medium" aria-live="polite">
            {labelPeriod(value)}
          </span>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Next period"
            disabled={!canGoNext(value, today)}
            onClick={() => { onChange(shiftPeriod(value, 1)); }}
          >
            <ChevronRight />
          </Button>
        </div>
      </div>
      {value.kind === "custom" ? <CustomRange value={value} max={today} onChange={onChange} /> : null}
    </div>
  );
}
