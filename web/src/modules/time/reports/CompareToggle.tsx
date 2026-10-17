import { SegmentedControl, type SegmentedOption } from "@/design";
import { COMPARE, COMPARE_MODES, type CompareMode } from "./compareMode";

const OPTIONS: readonly SegmentedOption[] = COMPARE_MODES.map((value) => ({ value, label: COMPARE[value].label }));

interface CompareToggleProps {
  compare: CompareMode;
  onChange: (compare: CompareMode) => void;
}

/** Chooses whether changes are measured against the Previous Period or the last 4 weeks. */
export function CompareToggle({ compare, onChange }: CompareToggleProps) {
  return (
    <SegmentedControl
      label="Compare with"
      value={compare}
      onValueChange={(value) => {
        onChange(value === "usual" ? "usual" : "previous");
      }}
      options={OPTIONS}
    />
  );
}
