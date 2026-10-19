import type { ReportGroup } from "@/api/time";
import { ColumnHeaderBar, SectionCard } from "@/design";
import { GroupRow } from "./GroupRow";
import type { RowContext } from "./rowContext";

interface SummaryTableProps {
  groups: readonly ReportGroup[];
  context: RowContext;
}

/** The groups of a Report, largest first. Every row starts closed; the header explains each column on hover or focus. */
export function SummaryTable({ groups, context }: SummaryTableProps) {
  const { grid, columns } = context.layout;
  const headings = [{ id: "group", label: "Group" }, ...columns.map((column) => ({ id: column.id, label: column.label, tooltip: column.tooltip, align: "end" as const }))];
  return (
    <SectionCard title="Summary">
      <ColumnHeaderBar grid={grid} columns={headings} />
      <ul className="rounded-b-xl">
        {groups.map((group) => (
          <GroupRow key={group.key || "none"} group={group} context={context} />
        ))}
      </ul>
    </SectionCard>
  );
}
