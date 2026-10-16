import { ChevronRight } from "lucide-react";
import { Fragment, useState } from "react";
import type { LifeLogProjectTotal } from "@/api/time";
import { cn } from "@/design";
import { formatCount } from "../format";
import { formatShare, formatSpan } from "../shared/duration";
import { CountsLine } from "./CountsLine";
import { DescriptionRow } from "./DescriptionRow";

interface ProjectRowProps {
  project: LifeLogProjectTotal;
  color: string;
  loggedSeconds: number;
}

/** One project's row in the totals table; opens to show its descriptions. */
export function ProjectRow({ project, color, loggedSeconds }: ProjectRowProps) {
  const [open, setOpen] = useState(false);
  return (
    <Fragment>
      <tr className="border-t">
        <td className="max-w-0 py-1.5">
          <button
            type="button"
            aria-expanded={open}
            onClick={() => { setOpen(!open); }}
            className="flex w-full min-w-0 items-center gap-1.5 rounded-md text-left font-medium hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring"
          >
            <ChevronRight aria-hidden className={cn("size-4 shrink-0 text-muted-foreground transition-transform motion-reduce:transition-none", open && "rotate-90")} />
            <span aria-hidden className="size-2.5 shrink-0 rounded-sm" style={{ background: color }} />
            <span className="min-w-0">
              <span className="block truncate" title={project.name}>{project.name}</span>
              <CountsLine entries={project.entries} days={project.days} />
            </span>
          </button>
        </td>
        <td className="py-1.5 pl-3 text-right text-sm whitespace-nowrap tabular-nums">{formatSpan(project.seconds)}</td>
        <td className="hidden py-1.5 pl-3 text-right text-sm tabular-nums sm:table-cell">{formatCount(project.entries)}</td>
        <td className="hidden py-1.5 pl-3 text-right text-sm tabular-nums sm:table-cell">{formatCount(project.days)}</td>
        <td className="py-1.5 pl-3 text-right text-xs text-muted-foreground tabular-nums">{formatShare(project.seconds, loggedSeconds)}</td>
      </tr>
      {open ? project.descriptions.map((total) => <DescriptionRow key={total.description} total={total} loggedSeconds={loggedSeconds} nested />) : null}
    </Fragment>
  );
}
