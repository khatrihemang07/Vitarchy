import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, type TooltipContentProps, XAxis, YAxis } from "recharts";
import type { ActivityPeriod } from "@/api/time";
import { axisProps, ChartLegend, gridProps, SEGMENT_GAP, tooltipProps } from "@/design";
import { ActivityPanel } from "./ActivityPanel";
import { secondsKey, trendChart } from "./chartModels";
import { ChartTable } from "../shared/ChartTable";
import { formatDayShort } from "./dayLabel";
import { formatSpan } from "./duration";
import { TrendTooltip } from "./TrendTooltip";
import { useActivityPart } from "./useActivityPart";

const MAX_BAR = 24;

/** Screen time week by week for the busiest apps. */
export function ActivityTrend({ period }: { period: ActivityPeriod }) {
  const query = useActivityPart(period, (overview) => overview.trend);
  return (
    <ActivityPanel
      title="Week by week"
      description="Hours per week for your five busiest apps. The first and last week may be partial."
      query={query}
      isEmpty={(data) => data.series.length === 0}
      emptyMessage="No Activity in this Period."
    >
      {(data) => {
        const { rows, series } = trendChart(data);
        return (
          <>
            <ChartLegend items={series.map((s) => ({ label: s.label, color: s.color }))} />
            <div role="img" aria-label="Stacked bar chart of hours per week by app. The same numbers follow as a table." className="h-64 w-full min-w-0">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={rows} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
                  <CartesianGrid {...gridProps} />
                  <XAxis dataKey="start" {...axisProps} tickFormatter={formatDayShort} interval="preserveStartEnd" minTickGap={24} />
                  <YAxis {...axisProps} width={52} tickFormatter={(h: number) => `${String(h)} h`} />
                  <Tooltip {...tooltipProps} content={(props: TooltipContentProps) => <TrendTooltip {...props} series={series} />} />
                  {series.map((s) => (
                    <Bar key={s.key} dataKey={s.key} stackId="week" fill={s.color} stroke="var(--card)" strokeWidth={SEGMENT_GAP / 2} maxBarSize={MAX_BAR} isAnimationActive={false} />
                  ))}
                </BarChart>
              </ResponsiveContainer>
            </div>
            <ChartTable
              caption="Hours per week by app"
              columns={["Week of", "Days", ...series.map((s) => s.label)]}
              rows={rows.map((row) => [formatDayShort(row.start), String(row.days), ...series.map((s) => formatSpan(Number(row[secondsKey(s.key)] ?? 0)))])}
            />
          </>
        );
      }}
    </ActivityPanel>
  );
}
