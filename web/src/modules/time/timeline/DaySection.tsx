import type { TimelineSummary } from "@/api/time";
import { formatSpan } from "../format";
import { type Clock } from "../reports/entryTimes";
import { periodLabel } from "../reports/period";
import { type AppColors } from "./appColors";
import { ColorLegend } from "./ColorLegend";
import { DayBand } from "./DayBand";
import { EntriesLane } from "./EntriesLane";
import { TimelineSection } from "./TimelineSection";

interface DaySectionProps {
  summary: TimelineSummary;
  colors: AppColors;
  clock: Clock;
  now: Date;
  highlight?: string | undefined;
  onApp: (app: string | undefined) => void;
}

/** Small label above a row of the Day's chart. */
const CAPTION = "mb-1 text-xs font-medium text-muted-foreground";

/** The Day's 24 h band with its Time Entries lane and colour legend. */
export function DaySection({ summary, colors, clock, now, highlight, onApp }: DaySectionProps) {
  const { overview, period } = summary;
  const segments = summary.segments ?? [];
  const label = `Timeline of ${periodLabel("day", period)}: ${formatSpan(overview.active_secs)} Active, ${formatSpan(overview.away_secs)} Away, ${formatSpan(overview.unknown_secs)} Away unknown, in ${String(segments.length)} Segments`;
  return (
    <TimelineSection title="The Day">
      <div className="flex flex-col gap-3">
        <div>
          <p className={CAPTION}>Time Entries</p>
          <EntriesLane day={period.from} entries={summary.entries ?? []} clock={clock} />
          <p className={`${CAPTION} mt-3`}>Activity</p>
          <DayBand day={period.from} segments={segments} colors={colors} clock={clock} highlight={highlight} now={now} summary={label} />
        </div>
        <ColorLegend apps={summary.apps} colors={colors} highlight={highlight} onApp={onApp} />
      </div>
    </TimelineSection>
  );
}
