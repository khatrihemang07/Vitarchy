import { m } from "motion/react";
import { useId, useState } from "react";
import type { ReportGroup } from "@/api/time";
import { cn, Expand } from "@/design";
import { GroupDetail } from "./GroupDetail";
import { GroupRowCells } from "./GroupRowCells";
import type { RowContext } from "./rowContext";

interface GroupRowProps {
  group: ReportGroup;
  context: RowContext;
  depth?: number;
}

const ROW_CLASSES = "grid w-full gap-x-3 gap-y-2 px-4 py-3 text-left";

/** One row: opens its second-level rows, or its chart and entries at the bottom level (never for a one-Day Report). */
export function GroupRow({ group, context, depth = 0 }: GroupRowProps) {
  const [open, setOpen] = useState(false);
  const panelId = useId();
  const hasChildren = group.children.length > 0;

  if (!hasChildren && context.isDay) {
    return (
      <li className="border-t border-border first:border-t-0 last:rounded-b-[inherit]">
        <div className={cn(ROW_CLASSES, context.layout.grid)}>
          <GroupRowCells group={group} context={context} depth={depth} open={undefined} />
        </div>
      </li>
    );
  }

  return (
    <li className="border-t border-border first:border-t-0 last:rounded-b-[inherit]">
      <m.button
        type="button"
        whileHover="hover"
        aria-expanded={open}
        aria-controls={panelId}
        onClick={() => {
          setOpen(!open);
        }}
        className={cn(ROW_CLASSES, context.layout.grid, "rounded-[inherit] transition-colors hover:bg-accent/50 focus-ring-inset aria-expanded:rounded-b-none")}
      >
        <GroupRowCells group={group} context={context} depth={depth} open={open} />
      </m.button>
      <Expand open={open} id={panelId}>
        {hasChildren ? (
          <ul className="rounded-b-[inherit] border-t border-border bg-muted/20">
            {group.children.map((child) => (
              <GroupRow key={child.key || "none"} group={child} context={context} depth={depth + 1} />
            ))}
          </ul>
        ) : (
          <GroupDetail group={group} clock={context.clock} />
        )}
      </Expand>
    </li>
  );
}
