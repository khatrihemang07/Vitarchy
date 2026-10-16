import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, type TooltipContentProps, XAxis, YAxis } from "recharts";
import { type ActivityPeriod, useLifeLogOverview } from "@/api/time";
import { axisProps, ChartLegend, gridProps, SEGMENT_GAP, tooltipProps } from "@/design";
import { ChartPanel } from "../shared/ChartPanel";
import { ChartTable } from "../shared/ChartTable";
import { formatDayShort } from "../shared/dayLabel";
import { formatSpan } from "../shared/duration";
import { SplitTooltip } from "./SplitTooltip";
import { hoursAxis, splitChart } from "./splitModel";
import { useProjectColors } from "./useProjectColors";

const MAX_BAR = 24;
const NO_COLORS: ReadonlyMap<string, string> = new Map();

/** How each Day divides between projects and what nobody logged, as stacked bars of hours. */
export function DaySplitChart({ period }: { period: ActivityPeriod }) {
  const query = useLifeLogOverview(period, (overview) => overview);
  const colors = useProjectColors(period) ?? NO_COLORS;
  return (
    <ChartPanel
      title="How your days divide"
      description="Hours per project each Day, and the time nobody logged. Overlapping entries each count in full, so a Day can pass 24 hours."
      query={query}
      isEmpty={(data) => data.totals.by_project.length === 0}
      emptyMessage="No Time Entries in this Period."
    >
      {(overview) => {
        const projects = overview.totals.by_project;
        const { rows, series } = splitChart(overview.split, projects, colors);
        const axis = hoursAxis(rows);
        return (
          <>
            <ChartLegend items={series.map((s) => ({ label: s.label, color: s.color }))} />
            <div role="img" aria-label="Stacked bar chart of hours per project for each Day. The same numbers follow as a table." className="h-64 w-full min-w-0">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={rows} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
                  <CartesianGrid {...gridProps} />
                  <XAxis dataKey="date" {...axisProps} tickFormatter={formatDayShort} interval="preserveStartEnd" minTickGap={24} />
                  <YAxis {...axisProps} width={52} domain={[0, axis.top]} ticks={axis.ticks} tickFormatter={(h: number) => `${String(h)} h`} />
                  <Tooltip {...tooltipProps} content={(props: TooltipContentProps) => <SplitTooltip {...props} series={series} />} />
                  {series.map((s) => (
                    <Bar key={s.key} dataKey={s.key} stackId="day" fill={s.color} stroke="var(--card)" strokeWidth={SEGMENT_GAP / 2} maxBarSize={MAX_BAR} isAnimationActive={false} />
                  ))}
                </BarChart>
              </ResponsiveContainer>
            </div>
            <ChartTable
              caption="Hours per project for each Day"
              columns={["Day", ...series.map((s) => s.label)]}
              rows={rows.map((row) => [formatDayShort(row.date), ...series.map((s) => formatSpan(row.seconds[s.key] ?? 0))])}
            />
          </>
        );
      }}
    </ChartPanel>
  );
}
