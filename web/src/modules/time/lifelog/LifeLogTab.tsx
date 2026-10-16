import type { PeriodBodyProps } from "../tabBodies";
import { browserTimeZone } from "../shared/browserTimeZone";
import { DaySplitChart } from "./DaySplitChart";
import { DescriptionStats } from "./DescriptionStats";
import { LifeLogDayTimeline } from "./LifeLogDayTimeline";
import { TotalsTable } from "./TotalsTable";

/** The Life log tab: how days divide, one day in detail, where logged time goes, and the usual routine. */
export default function LifeLogTab({ selection }: PeriodBodyProps) {
  const period = { from: selection.from, to: selection.to, tz: browserTimeZone() };
  return (
    <div className="flex min-w-0 flex-col gap-6">
      <DaySplitChart period={period} />
      <LifeLogDayTimeline period={period} />
      <TotalsTable period={period} />
      <DescriptionStats period={period} />
    </div>
  );
}
