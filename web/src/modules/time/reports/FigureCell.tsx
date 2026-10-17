import { formatSpan } from "../format";
import { ChangeBadge } from "./ChangeBadge";
import type { Figure } from "./groupFigures";
import { StatCell } from "./StatCell";

interface FigureCellProps {
  label: string;
  figure: Figure;
  /** Bold, for the total. */
  emphasis?: boolean;
}

/** A duration with its change from the Previous Period beneath. */
export function FigureCell({ label, figure, emphasis = false }: FigureCellProps) {
  return (
    <StatCell label={label}>
      <span className={emphasis ? "font-semibold" : undefined}>{formatSpan(figure.value)}</span>
      <ChangeBadge delta={figure.change} unit="seconds" />
    </StatCell>
  );
}
