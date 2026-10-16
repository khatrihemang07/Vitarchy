import { BarRow } from "../shared/BarRow";
import { formatShare, formatSpan, fractionOf } from "../shared/duration";
import type { NamedSeconds } from "./overviewModel";

interface CompactBarsProps {
  /** The rows, longest first. */
  rows: readonly NamedSeconds[];
  /** What the shares are of. */
  total: number;
  colorOf: (name: string) => string;
  label: string;
}

/** A short list of names with a thin bar, the time and its share: the Overview's compact chart. */
export function CompactBars({ rows, total, colorOf, label }: CompactBarsProps) {
  const longest = rows[0]?.seconds ?? 0;
  return (
    <ul aria-label={label} className="flex flex-col">
      {rows.map((row) => (
        <BarRow key={row.name} label={row.name} value={formatSpan(row.seconds)} share={formatShare(row.seconds, total)} fraction={fractionOf(row.seconds, longest)} color={colorOf(row.name)} />
      ))}
    </ul>
  );
}
