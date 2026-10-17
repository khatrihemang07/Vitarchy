import { SegmentedControl, type SegmentedOption } from "@/design";
import type { StatMode } from "./reportSearch";

const OPTIONS: readonly SegmentedOption[] = [
  { value: "per_day", label: "Per Day" },
  { value: "per_entry", label: "Per entry" },
];

interface StatModeToggleProps {
  mode: StatMode;
  onChange: (mode: StatMode) => void;
}

/** Chooses whether average, least and most are measured per Day or per Time Entry. */
export function StatModeToggle({ mode, onChange }: StatModeToggleProps) {
  return (
    <SegmentedControl
      label="Measure"
      value={mode}
      onValueChange={(value) => {
        onChange(value === "per_entry" ? "per_entry" : "per_day");
      }}
      options={OPTIONS}
    />
  );
}
