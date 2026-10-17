import type { Cell } from "./columns";
import { ChangeBadge } from "./ChangeBadge";
import type { CompareMode } from "./compareMode";
import { StatCell } from "./StatCell";

interface ColumnCellProps {
  label: string;
  cell: Cell;
  compare: CompareMode;
}

/** One cell of a summary row: its text, with the change from the chosen comparison beneath. */
export function ColumnCell({ label, cell, compare }: ColumnCellProps) {
  return (
    <StatCell label={label}>
      <span className={cell.emphasis ? "font-semibold" : undefined}>{cell.text}</span>
      <ChangeBadge delta={cell.change ?? null} unit="seconds" against={compare} />
    </StatCell>
  );
}
