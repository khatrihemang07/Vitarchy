import type { ReportGroup } from "@/api/time";
import { DailyBars } from "@/design";
import { formatSpan } from "../format";
import { type Clock } from "./entryTimes";
import { EntryList } from "./EntryList";

interface GroupDetailProps {
  group: ReportGroup;
  clock: Clock;
}

/** What opens under a bottom-level row: its time per Day, then its Time Entries. */
export function GroupDetail({ group, clock }: GroupDetailProps) {
  return (
    <div className="flex flex-col gap-3 bg-muted/30 py-3">
      <DailyBars
        className="px-4"
        label={`Time per Day for ${group.label}`}
        data={group.daily.map((d) => ({ date: d.date, seconds: d.seconds }))}
        formatValue={formatSpan}
      />
      <EntryList entries={group.entries} clock={clock} />
    </div>
  );
}
