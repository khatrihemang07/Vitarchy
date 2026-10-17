import type { ReportGroup } from "@/api/time";
import { figureLabels, ROW_GRID } from "./columns";
import { GroupRow } from "./GroupRow";
import type { RowContext } from "./rowContext";

interface SummaryTableProps {
  groups: readonly ReportGroup[];
  context: RowContext;
}

/** The groups of a Report, largest first; the first opens by default (with its first child, when grouped twice) so the page never looks empty. */
export function SummaryTable({ groups, context }: SummaryTableProps) {
  const labels = figureLabels(context.mode);
  const headings = ["Days", "Entries", labels.total, labels.avg, labels.min, labels.max];
  return (
    <section aria-label="Summary" className="overflow-hidden rounded-xl border border-border bg-card">
      <div aria-hidden="true" className={`${ROW_GRID} hidden gap-x-3 border-b border-border bg-muted/40 px-4 py-2 text-xs font-medium text-muted-foreground`}>
        <span>Group</span>
        {headings.map((heading) => (
          <span key={heading} className="text-right">
            {heading}
          </span>
        ))}
      </div>
      <ul>
        {groups.map((group, index) => (
          <GroupRow key={group.key || "none"} group={group} context={context} defaultOpen={index === 0} />
        ))}
      </ul>
    </section>
  );
}
