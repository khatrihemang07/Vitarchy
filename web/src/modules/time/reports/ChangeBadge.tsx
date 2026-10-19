import { ChangeBadge as ChangeBadgeView } from "@/design";
import { COMPARE, type CompareMode } from "./compareMode";
import { type ChangeUnit, formatChange } from "./formatChange";

interface ChangeBadgeProps {
  /** Current minus the reference; null when there is nothing to compare with. */
  delta: number | null;
  unit: ChangeUnit;
  /** What the figure was compared with. */
  against: CompareMode;
}

/** How a figure moved against the Previous Period or the Usual, in time or a count. */
export function ChangeBadge({ delta, unit, against }: ChangeBadgeProps) {
  return <ChangeBadgeView delta={delta} text={delta === null ? "" : formatChange(delta, unit)} against={COMPARE[against].phrase} />;
}
