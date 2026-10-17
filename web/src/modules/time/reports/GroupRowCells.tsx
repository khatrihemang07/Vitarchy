import { ChevronRight } from "lucide-react";
import { m } from "motion/react";
import type { ReportGroup } from "@/api/time";
import { Badge, cn, nudge } from "@/design";
import { ColumnCell } from "./ColumnCell";
import { figuresFor } from "./groupFigures";
import type { RowContext } from "./rowContext";

const INDENT_REM = 1.25;

interface GroupRowCellsProps {
  group: ReportGroup;
  context: RowContext;
  depth: number;
  /** Whether the row is open; undefined when it cannot open, so it shows no chevron. */
  open: boolean | undefined;
}

/** The label and cells of one row, shared by rows that open and rows that only show their stats. */
export function GroupRowCells({ group, context, depth, open }: GroupRowCellsProps) {
  const figures = figuresFor(group, context.mode, context.compare);
  const label = group.label || "(none)";
  return (
    <>
      <m.span
        variants={{ hover: nudge.whileHover }}
        transition={nudge.transition}
        className="col-span-full flex min-w-0 items-center gap-2 md:col-span-1"
        style={{ paddingLeft: `${String(depth * INDENT_REM)}rem` }}>
        {open === undefined ? (
          // Only a nested row keeps the chevron's space, so its label lines up under its parent's.
          depth > 0 ? <span className="size-4 shrink-0" aria-hidden="true" /> : null
        ) : (
          <ChevronRight className={cn("size-4 shrink-0 text-muted-foreground transition-transform", open && "rotate-90")} aria-hidden="true" />
        )}
        <span className="min-w-0 truncate font-medium" title={label}>
          {label}
        </span>
        {figures.isNew ? <Badge variant="outline">new</Badge> : null}
      </m.span>
      {context.layout.columns.map((column) => (
        <ColumnCell key={column.id} label={column.label} cell={column.cell(figures, context)} compare={context.compare} />
      ))}
    </>
  );
}
