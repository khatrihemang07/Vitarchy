import type { ActivityPeriod } from "@/api/time";
import { ChartPanel } from "../shared/ChartPanel";
import { TotalsList } from "./TotalsList";
import { useActivityPart } from "./useActivityPart";

/** Where screen time goes: apps, then the project or site inside each, then titles. */
export function ActivityTotals({ period }: { period: ActivityPeriod }) {
  const query = useActivityPart(period, (overview) => overview.totals);
  return (
    <ChartPanel
      title="Where your screen time goes"
      description="Every app, counted as recorded. Open one to see what was in it."
      query={query}
      isEmpty={(data) => data.apps.length === 0}
      emptyMessage="No Activity in this Period."
    >
      {(totals) => <TotalsList totals={totals} />}
    </ChartPanel>
  );
}
