import { ActivityDayBounds } from "./ActivityDayBounds";
import { ActivityDayTimeline } from "./ActivityDayTimeline";
import { ActivityHeatmap } from "./ActivityHeatmap";
import { ActivityTotals } from "./ActivityTotals";
import { ActivityTrend } from "./ActivityTrend";
import { browserTimeZone } from "./browserTimeZone";
import type { PeriodBodyProps } from "../tabBodies";

/** The Activity tab: where screen time goes, when it happens, and how it changes. */
export default function ActivityTab({ selection }: PeriodBodyProps) {
  const period = { from: selection.from, to: selection.to, tz: browserTimeZone() };
  return (
    <div className="flex min-w-0 flex-col gap-6">
      <ActivityTotals period={period} />
      <ActivityHeatmap period={period} />
      <ActivityDayBounds period={period} />
      <ActivityDayTimeline period={period} />
      <ActivityTrend period={period} />
    </div>
  );
}
