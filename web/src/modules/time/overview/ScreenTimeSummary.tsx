import { Minus, TrendingDown, TrendingUp } from "lucide-react";
import type { ActivityOverview, ActivityPeriod, ActivityTotals } from "@/api/time";
import { seriesColor } from "@/design";
import { useActivityPart } from "../activity/useActivityPart";
import { ChartPanel } from "../shared/ChartPanel";
import { formatSpan } from "../shared/duration";
import { CompactBars } from "./CompactBars";
import { type Change, describeChange, topApps, topContexts } from "./overviewModel";

/** Apps and Contexts listed on the Overview. */
const TOP_COUNT = 5;

const CHANGE_ICONS = { up: TrendingUp, down: TrendingDown, same: Minus } as const;

// Defined once so the query memoizes the selection.
const selectTotals = (overview: ActivityOverview) => overview.totals;

interface ScreenTimeSummaryProps {
  period: ActivityPeriod;
  /** The Period right before, for the comparison. */
  previous: ActivityPeriod;
}

/** Screen time for the Period: the total, how it compares with the Period before, and the top apps and Contexts. */
export function ScreenTimeSummary({ period, previous }: ScreenTimeSummaryProps) {
  const query = useActivityPart(period, selectTotals);
  const before = useActivityPart(previous, selectTotals);
  return (
    <ChartPanel
      title="Screen time"
      description="Computer Activity, counted as recorded."
      query={query}
      isEmpty={(totals) => totals.total_seconds === 0}
      emptyMessage="No Activity in this Period."
    >
      {(totals) => <ScreenTimeBody totals={totals} change={before.data ? describeChange(totals.total_seconds, before.data.total_seconds) : null} />}
    </ChartPanel>
  );
}

function ScreenTimeBody({ totals, change }: { totals: ActivityTotals; change: Change | null }) {
  const ChangeIcon = change ? CHANGE_ICONS[change.direction] : null;
  return (
    <>
      <div>
        <p className="text-3xl font-semibold tabular-nums">{formatSpan(totals.total_seconds)}</p>
        {change && ChangeIcon ? (
          <p className="flex items-center gap-1.5 text-sm text-muted-foreground">
            <ChangeIcon aria-hidden className="size-4" />
            {change.text}
          </p>
        ) : null}
      </div>
      <div className="flex flex-col gap-4">
        <section aria-label="Top apps" className="min-w-0">
          <h3 className="text-sm font-medium text-muted-foreground">Top apps</h3>
          <CompactBars label="Top apps" rows={topApps(totals, TOP_COUNT)} total={totals.total_seconds} colorOf={() => seriesColor(0)} />
        </section>
        <section aria-label="Top Contexts" className="min-w-0">
          <h3 className="text-sm font-medium text-muted-foreground">Top Contexts</h3>
          <CompactBars label="Top Contexts" rows={topContexts(totals, TOP_COUNT)} total={totals.total_seconds} colorOf={() => seriesColor(1)} />
        </section>
      </div>
    </>
  );
}
