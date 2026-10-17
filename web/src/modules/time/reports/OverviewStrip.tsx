import type { Report } from "@/api/time";
import { Card, DailyBars, NumberTicker, StatCard } from "@/design";
import { formatCount, formatSpan } from "../format";
import { ChangeBadge } from "./ChangeBadge";
import type { ChangeUnit } from "./formatChange";
import { periodLabel } from "./period";

interface TileProps {
  title: string;
  value: number;
  format: (value: number) => string;
  change: number;
  unit: ChangeUnit;
}

/** One figure that counts up to its value; screen readers get the final text at once. */
function Tile({ title, value, format, change, unit }: TileProps) {
  return (
    <StatCard
      label={title}
      value={
        <>
          <NumberTicker value={value} format={format} className="text-foreground tracking-tight" aria-hidden="true" />
          <span className="sr-only">{format(value)}</span>
        </>
      }
      badge={<ChangeBadge delta={change} unit={unit} />}
    />
  );
}

/** The Period at a glance: total, Days tracked and average, each against the Previous Period, over the daily bars. */
export function OverviewStrip({ report }: { report: Report }) {
  const { overview, period, daily } = report;
  const previous = periodLabel(period.kind, { from: period.previous_from, to: period.previous_to });
  return (
    <section aria-labelledby="overview-title" className="flex flex-col gap-3">
      <h2 id="overview-title" className="font-display text-xl font-bold">Overview</h2>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <Tile title="Total" value={overview.total_s} format={formatSpan} change={overview.change.total_s} unit="seconds" />
        <Tile title="Days tracked" value={overview.days_tracked} format={formatCount} change={overview.change.days_tracked} unit="count" />
        <Tile title="Average per tracked Day" value={overview.avg_per_day_s} format={formatSpan} change={overview.change.avg_per_day_s} unit="seconds" />
      </div>
      <p className="text-xs text-muted-foreground">Changes compare with the Previous Period, {previous}.</p>
      {period.kind === "day" ? null : (
        <Card className="p-4 md:p-5">
          <DailyBars label="Time per Day" data={daily.map((d) => ({ date: d.date, seconds: d.seconds }))} formatValue={formatSpan} />
        </Card>
      )}
    </section>
  );
}
