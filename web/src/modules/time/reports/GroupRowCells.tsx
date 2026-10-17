import { ChevronRight } from "lucide-react";
import { m } from "motion/react";
import type { ReportGroup } from "@/api/time";
import { Badge, cn, nudge } from "@/design";
import { formatCount } from "../format";
import { figureLabels } from "./columns";
import { FigureCell } from "./FigureCell";
import { figuresFor } from "./groupFigures";
import type { RowContext } from "./rowContext";
import { StatCell } from "./StatCell";

const INDENT_REM = 1.25;

interface GroupRowCellsProps {
  group: ReportGroup;
  context: RowContext;
  depth: number;
  /** Whether the row is open; undefined when it cannot open, which leaves the chevron's space empty. */
  open: boolean | undefined;
}

/** The label and figures of one row, shared by rows that open and rows that only show their stats. */
export function GroupRowCells({ group, context, depth, open }: GroupRowCellsProps) {
  const figures = figuresFor(group, context.mode);
  const labels = figureLabels(context.mode);
  const label = group.label || "(none)";
  return (
    <>
      <m.span
        variants={{ hover: nudge.whileHover }}
        transition={nudge.transition}
        className="col-span-3 flex min-w-0 items-center gap-2 md:col-span-1"
        style={{ paddingLeft: `${String(depth * INDENT_REM)}rem` }}>
        {open === undefined ? (
          <span className="size-4 shrink-0" aria-hidden="true" />
        ) : (
          <ChevronRight className={cn("size-4 shrink-0 text-muted-foreground transition-transform", open && "rotate-90")} aria-hidden="true" />
        )}
        <span className="min-w-0 truncate font-medium" title={label}>
          {label}
        </span>
        {figures.isNew ? <Badge variant="outline">new</Badge> : null}
      </m.span>
      <StatCell label="Days">
        {formatCount(figures.days)} of {formatCount(context.periodDays)}
      </StatCell>
      <StatCell label="Entries">{formatCount(figures.entries)}</StatCell>
      <FigureCell label={labels.total} figure={figures.total} emphasis />
      <FigureCell label={labels.avg} figure={figures.avg} />
      <FigureCell label={labels.min} figure={figures.min} />
      <FigureCell label={labels.max} figure={figures.max} />
    </>
  );
}
