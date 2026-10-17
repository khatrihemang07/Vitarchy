import { Minus, TrendingDown, TrendingUp } from "lucide-react";
import type { ActivityOverview, ActivityPeriod } from "@/api/time";
import { seriesColor } from "@/design";
import { useActivityPart } from "../activity/useActivityPart";
import { ChartPanel } from "../shared/ChartPanel";
import { formatSpan } from "../shared/duration";
import { CompactBars } from "./CompactBars";
import { type Change, describeChange, topApps } from "./overviewModel";

/** Apps listed on the Overview (the server decides how many Contexts). */
const TOP_APPS = 5;

const CHANGE_ICONS = { up: TrendingUp, down: TrendingDown, same: Minus } as const;

// Defined once so the query memoizes the selection.
const selectOverview = (overview: ActivityOverview) => overview;
const selectDayTotals = (overview: ActivityOverview) => overview.day_totals;

interface ScreenTimeSummaryProps {
  period: ActivityPeriod;
  /** The Period right before, for the comparison. */
  previous: ActivityPeriod;
}

/** Screen time for the Period: the total, how it compares with the Period before, and the top apps and Contexts. */
export function ScreenTimeSummary({ period, previous }: ScreenTimeSummaryProps) {
  const query = useActivityPart(period, selectOverview);
  const before = useActivityPart(previous, selectDayTotals);
  return (
    <ChartPanel
      title="Screen time"
      description="Computer Activity, counted as recorded."
      query={query}
      isEmpty={(overview) => overview.totals.total_seconds === 0}
      emptyMessage="No Activity in this Period."
    >
      {(overview) => <ScreenTimeBody overview={overview} change={before.data ? describeChange(overview.day_totals, before.data) : null} />}
    </ChartPanel>
  );
}

function ScreenTimeBody({ overview, change }: { overview: ActivityOverview; change: Change | null }) {
  const { totals } = overview;
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
          <CompactBars label="Top apps" rows={topApps(totals, TOP_APPS)} total={totals.total_seconds} colorOf={() => seriesColor(0)} />
        </section>
        <section aria-label="Top Contexts" className="min-w-0">
          <h3 className="text-sm font-medium text-muted-foreground">Top Contexts</h3>
          <CompactBars label="Top Contexts" rows={overview.top_contexts} total={totals.total_seconds} colorOf={() => seriesColor(1)} />
        </section>
      </div>
    </>
  );
}
