import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button, Input } from "@/design";
import { addDays } from "./dates";

interface DayPickerProps {
  /** The chosen local date, `YYYY-MM-DD`. */
  day: string;
  /** The first and last Day that may be chosen. */
  from: string;
  to: string;
  onChange: (day: string) => void;
}

/** Choose one Day of the Period: step by a day or type a date. */
export function DayPicker({ day, from, to, onChange }: DayPickerProps) {
  return (
    <div className="flex items-center gap-1">
      <Button variant="ghost" size="icon" aria-label="Previous day" disabled={addDays(day, -1) < from} onClick={() => { onChange(addDays(day, -1)); }}>
        <ChevronLeft />
      </Button>
      <Input
        type="date"
        aria-label="Day"
        className="w-40"
        value={day}
        min={from}
        max={to}
        onChange={(event) => {
          if (event.target.value) onChange(event.target.value);
        }}
      />
      <Button variant="ghost" size="icon" aria-label="Next day" disabled={addDays(day, 1) > to} onClick={() => { onChange(addDays(day, 1)); }}>
        <ChevronRight />
      </Button>
    </div>
  );
}
