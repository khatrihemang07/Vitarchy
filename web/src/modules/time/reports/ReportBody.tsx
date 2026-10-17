import type { Report } from "@/api/time";
import { FadeUp, MotionButton } from "@/design";
import { TimeNotice } from "../TimeNotice";
import { DayEntries } from "./DayEntries";
import { elapsedDays } from "./elapsedDays";
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
  /** Place in the page's entrance sequence of the first section, after whatever sits above. */
  firstStagger: number;
}

/** A loaded Report: overview, then (for one Day) its entries, then the summary table, or the empty notice. */
export function ReportBody({ report, mode, clock, today, filtered, onClear, firstStagger }: ReportBodyProps) {
  if (report.groups.length === 0) {
    return (
      <FadeUp index={firstStagger}>
        <TimeNotice
          kind="empty"
          title={filtered ? "No Time Entries match" : "No Time Entries in this Period"}
          action={
            filtered ? (
              <MotionButton variant="outline" onClick={onClear}>
                Clear filters
              </MotionButton>
            ) : undefined
          }
        >
          {filtered ? "Try other filters, or another Period." : "Try another Period, or Sync from the Time page."}
        </TimeNotice>
      </FadeUp>
    );
  }
  const isDay = report.period.kind === "day";
  const context = { mode, clock, periodDays: elapsedDays(report.period, today), isDay };
  const sections = [
    <OverviewStrip key="overview" report={report} />,
    ...(isDay ? [<DayEntries key="entries" report={report} clock={clock} />] : []),
    <SummaryTable key={`summary-${report.group1}/${report.group2 ?? ""}`} groups={report.groups} context={context} />,
  ];
  return (
    <div className="flex flex-col gap-6">
      {sections.map((section, index) => (
        <FadeUp key={section.key} index={firstStagger + index}>
          {section}
        </FadeUp>
      ))}
    </div>
  );
}
