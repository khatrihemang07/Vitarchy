import type { Report } from "@/api/time";
import { type Clock, chronologicalEntries } from "./entryTimes";
import { EntryList } from "./EntryList";

/** A one-Day Report lists its Time Entries in order, instead of a chart of one bar per group. */
export function DayEntries({ report, clock }: { report: Report; clock: Clock }) {
  return (
    <section aria-label="Time Entries of the Day" className="overflow-hidden rounded-xl border border-border bg-card">
      <h2 className="border-b border-border px-4 py-2 text-sm font-medium text-muted-foreground">Time Entries</h2>
      <EntryList entries={chronologicalEntries(report.groups)} clock={clock} />
    </section>
  );
}
