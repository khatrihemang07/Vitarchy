import type { Report } from "@/api/time";
import { DailyBars } from "@/design";
import { formatCount, formatSpan } from "../format";
import { periodLabel } from "./period";
import { ChangeBadge } from "./ChangeBadge";
import type { ChangeUnit } from "./formatChange";

interface TileProps {
  title: string;
  value: string;
  change: number;
  unit: ChangeUnit;
}

function Tile({ title, value, change, unit }: TileProps) {
  return (
    <div className="flex min-w-0 flex-col gap-1">
      <dt className="text-sm text-muted-foreground">{title}</dt>
      <dd className="flex flex-wrap items-center gap-x-2 gap-y-1">
        <span className="font-display text-3xl font-bold tracking-tight tabular-nums">{value}</span>
        <ChangeBadge delta={change} unit={unit} />
      </dd>
    </div>
  );
}

interface OverviewStripProps {
  report: Report;
}

/** The Period at a glance: total, Days tracked and average, each against the Previous Period, over the daily bars. */
export function OverviewStrip({ report }: OverviewStripProps) {
  const { overview, period, daily } = report;
  return (
    <section aria-label="Overview" className="flex flex-col gap-4 rounded-xl border border-border bg-card p-4 md:p-5">
      <dl className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <Tile title="Total" value={formatSpan(overview.total_s)} change={overview.change.total_s} unit="seconds" />
        <Tile title="Days tracked" value={formatCount(overview.days_tracked)} change={overview.change.days_tracked} unit="count" />
        <Tile title="Average per tracked Day" value={formatSpan(overview.avg_per_day_s)} change={overview.change.avg_per_day_s} unit="seconds" />
      </dl>
      <p className="text-xs text-muted-foreground">
        Changes compare with the Previous Period, {periodLabel(period.kind, { from: period.previous_from, to: period.previous_to })}.
      </p>
      {period.kind === "day" ? null : (
        <DailyBars label="Time per Day" data={daily.map((d) => ({ date: d.date, seconds: d.seconds }))} formatValue={formatSpan} />
      )}
    </section>
  );
}
