import type { Report } from "@/api/time";
import { DayEntries } from "./DayEntries";
import { elapsedDays } from "./elapsedDays";
import { EmptyReport } from "./EmptyReport";
import type { Clock } from "./entryTimes";
import { OverviewStrip } from "./OverviewStrip";
import type { StatMode } from "./reportSearch";
import { SummaryTable } from "./SummaryTable";

interface ReportBodyProps {
  report: Report;
  mode: StatMode;
  clock: Clock;
  /** Today's local Day, so "of M" leaves out Days that have not happened. */
  today: string;
  filtered: boolean;
  onClear: () => void;
}

/** A loaded Report: overview, then (for one Day) its entries, then the summary table, or the empty notice. */
export function ReportBody({ report, mode, clock, today, filtered, onClear }: ReportBodyProps) {
  if (report.groups.length === 0) return <EmptyReport filtered={filtered} onClear={onClear} />;
  const isDay = report.period.kind === "day";
  const context = { mode, clock, periodDays: elapsedDays(report.period, today), isDay };
  return (
    <div className="flex flex-col gap-4">
      <OverviewStrip report={report} />
      {isDay ? <DayEntries report={report} clock={clock} /> : null}
      <SummaryTable key={`${report.group1}/${report.group2 ?? ""}`} groups={report.groups} context={context} />
    </div>
  );
}
