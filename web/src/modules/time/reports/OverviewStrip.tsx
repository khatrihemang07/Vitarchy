import type { Report } from "@/api/time";
import { Card, DailyBars, FadeValue, StatCard } from "@/design";
import { formatCount, formatSpan } from "../format";
import { ChangeBadge } from "./ChangeBadge";
import { COMPARE, type CompareMode, compareNote } from "./compareMode";
import type { ChangeUnit } from "./formatChange";

interface TileProps {
  title: string;
  value: number;
  format: (value: number) => string;
  change: number;
  unit: ChangeUnit;
  against: CompareMode;
}

/** One figure that fades in when it changes (no counting up), with its change beneath. */
function Tile({ title, value, format, change, unit, against }: TileProps) {
  return (
    <StatCard
      label={title}
      value={<FadeValue text={format(value)} className="text-foreground tracking-tight" />}
      badge={<ChangeBadge delta={change} unit={unit} against={against} />}
    />
  );
}

/** The Period at a glance: total, Days tracked and average, each against the chosen comparison, over the daily bars. */
export function OverviewStrip({ report, compare }: { report: Report; compare: CompareMode }) {
  const { overview, period, daily } = report;
  const change = COMPARE[compare].overview(overview);
  return (
    <section aria-labelledby="overview-title" className="flex flex-col gap-3">
      <h2 id="overview-title" className="font-display text-xl font-bold">Overview</h2>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <Tile title="Total" value={overview.total_s} format={formatSpan} change={change.total_s} unit="seconds" against={compare} />
        <Tile title="Days tracked" value={overview.days_tracked} format={formatCount} change={change.days_tracked} unit="count" against={compare} />
        <Tile title="Average per tracked Day" value={overview.avg_per_day_s} format={formatSpan} change={change.avg_per_day_s} unit="seconds" against={compare} />
      </div>
      <p className="text-xs text-muted-foreground">{compareNote(report, compare)}</p>
      {period.kind === "day" ? null : (
        <Card className="p-4 md:p-5">
          <DailyBars label="Time per Day" data={daily.map((d) => ({ date: d.date, seconds: d.seconds }))} formatValue={formatSpan} />
        </Card>
      )}
    </section>
  );
}
