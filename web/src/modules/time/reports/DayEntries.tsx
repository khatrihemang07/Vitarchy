import type { Report } from "@/api/time";
import { type Clock, chronologicalEntries } from "./entryTimes";
import { EntryList } from "./EntryList";
import { ReportCard } from "./ReportCard";

/** A one-Day Report lists its Time Entries in order, instead of a chart of one bar per group. */
export function DayEntries({ report, clock }: { report: Report; clock: Clock }) {
  return (
    <ReportCard title="Time Entries">
      <EntryList entries={chronologicalEntries(report.groups)} clock={clock} />
    </ReportCard>
  );
}
