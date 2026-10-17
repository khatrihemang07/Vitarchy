import { useState } from "react";
import type { ReportEntry } from "@/api/time";
import { cn } from "@/design";
import { formatSpan } from "../format";
import { type Clock, describeEntry } from "./entryTimes";

/** How many entries show before "Show all"; a month of one project can run to hundreds. */
const INITIAL_LIMIT = 50;

interface EntryListProps {
  entries: readonly ReportEntry[];
  clock: Clock;
}

function EntryRow({ entry, clock }: { entry: ReportEntry; clock: Clock }) {
  const times = describeEntry(entry, clock);
  return (
    <li className="grid grid-cols-[1fr_auto] gap-x-3 gap-y-0.5 px-4 py-2 text-sm md:grid-cols-[6.5rem_14rem_minmax(0,1fr)_auto] md:items-baseline">
      <span className="text-muted-foreground tabular-nums">{times.date}</span>
      <span className="order-last col-span-2 tabular-nums md:order-none md:col-span-1">
        {times.start} – {times.stop}
        {times.crossesMidnight ? (
          <span className="ml-1.5 whitespace-nowrap rounded-full bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">crosses midnight</span>
        ) : null}
      </span>
      <span className="order-last col-span-2 min-w-0 md:order-none md:col-span-1">
        <span className={cn("block truncate", !entry.description && "text-muted-foreground")}>{entry.description || "(no description)"}</span>
        <span className="block truncate text-xs text-muted-foreground">{[entry.project_name ?? "(no project)", ...entry.tags].join(" · ")}</span>
      </span>
      <span className="text-right font-medium tabular-nums">{formatSpan(entry.duration_s)}</span>
    </li>
  );
}

/** The Time Entries of a group: Day, start to stop, project, tags and the time inside the Period. */
export function EntryList({ entries, clock }: EntryListProps) {
  const [showAll, setShowAll] = useState(false);
  const shown = showAll ? entries : entries.slice(0, INITIAL_LIMIT);
  return (
    <div>
      <ul className="divide-y divide-border">
        {shown.map((entry) => (
          <EntryRow key={entry.toggl_id} entry={entry} clock={clock} />
        ))}
      </ul>
      {shown.length < entries.length ? (
        <button
          type="button"
          onClick={() => {
            setShowAll(true);
          }}
          className="w-full px-4 py-2 text-left text-sm text-primary hover:underline focus-ring"
        >
          Show all {entries.length} entries
        </button>
      ) : null}
    </div>
  );
}
