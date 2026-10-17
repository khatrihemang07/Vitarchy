import type { ReportGroup } from "@/api/time";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/design";
import { GroupRow } from "./GroupRow";
import { ReportCard } from "./ReportCard";
import type { RowContext } from "./rowContext";

interface SummaryTableProps {
  groups: readonly ReportGroup[];
  context: RowContext;
}

/** The groups of a Report, largest first. Every row starts closed; the header explains each column on hover or focus. */
export function SummaryTable({ groups, context }: SummaryTableProps) {
  const { grid, columns } = context.layout;
  return (
    <ReportCard title="Summary">
      <div className={`${grid} hidden gap-x-3 border-b border-border bg-muted/40 px-4 py-2 text-xs font-medium text-muted-foreground`}>
        <span>Group</span>
        {columns.map((column) => (
          <Tooltip key={column.id}>
            <TooltipTrigger asChild>
              <span tabIndex={0} className="cursor-help justify-self-end rounded-sm text-right underline decoration-dotted decoration-muted-foreground/50 underline-offset-4 focus-ring">
                {column.label}
              </span>
            </TooltipTrigger>
            <TooltipContent>{column.tooltip}</TooltipContent>
          </Tooltip>
        ))}
      </div>
      <ul className="rounded-b-xl">
        {groups.map((group) => (
          <GroupRow key={group.key || "none"} group={group} context={context} />
        ))}
      </ul>
    </ReportCard>
  );
}
