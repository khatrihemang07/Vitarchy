import { useCoreSettings } from "@/app/module-manifest";
import { previousPeriod, selectionFor } from "../period";
import { browserTimeZone } from "../shared/browserTimeZone";
import type { PeriodBodyProps } from "../tabBodies";
import { InsightList } from "./InsightList";
import type { InsightTarget } from "./insightTarget";
import { LifeLogSummary } from "./LifeLogSummary";
import { ScreenTimeSummary } from "./ScreenTimeSummary";

/** The Overview tab: the most notable Insights, then where screen time goes and how the Life log looks. */
export default function OverviewTab({ selection, open }: PeriodBodyProps) {
  const weekStart = useCoreSettings().week_start;
  const timeZone = browserTimeZone();
  const period = { from: selection.from, to: selection.to, tz: timeZone };
  const before = previousPeriod(selection);
  const previous = { from: before.from, to: before.to, tz: timeZone };
  // An Insight about one Day opens that Day; any other keeps the Period being looked at.
  const openTarget = ({ tab, day }: InsightTarget) => { open(tab, day === undefined ? selection : selectionFor("day", day, weekStart)); };
  return (
    <div className="flex min-w-0 flex-col gap-6">
      <InsightList period={period} onOpen={openTarget} />
      <div className="grid min-w-0 gap-6 lg:grid-cols-2">
        <ScreenTimeSummary period={period} previous={previous} />
        <LifeLogSummary period={period} />
      </div>
    </div>
  );
}
