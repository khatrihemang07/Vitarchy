import type { Report } from "@/api/time";
import { ClearFiltersButton, FadeUp, Notice } from "@/design";
import { DayEntries } from "./DayEntries";
import type { Clock } from "./entryTimes";
import { OverviewStrip } from "./OverviewStrip";
import type { CompareMode } from "./compareMode";
import { rowContextFor } from "./rowContext";
import type { StatMode } from "./reportSearch";
import { SummaryTable } from "./SummaryTable";

interface ReportBodyProps {
  report: Report;
  mode: StatMode;
  compare: CompareMode;
  clock: Clock;
  filtered: boolean;
  onClear: () => void;
  /** Place in the page's entrance sequence of the first section, after whatever sits above. */
  firstStagger: number;
}

/** A loaded Report: overview, then (for one Day) its entries, then the summary table, or the empty notice. */
export function ReportBody({ report, mode, compare, clock, filtered, onClear, firstStagger }: ReportBodyProps) {
  if (report.groups.length === 0) {
    return (
      <FadeUp index={firstStagger}>
        <Notice
          kind="empty"
          title={filtered ? "No Time Entries match" : "No Time Entries in this Period"}
          action={
            filtered ? <ClearFiltersButton onClear={onClear} /> : undefined
          }
        >
          {filtered ? "Try other filters, or another Period." : "Try another Period, or Sync from the Time page."}
        </Notice>
      </FadeUp>
    );
  }
  const isDay = report.period.kind === "day";
  const context = rowContextFor({ period: report.period, mode, compare, clock });
  const sections = [
    <OverviewStrip key="overview" report={report} compare={compare} />,
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
