import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, type TooltipContentProps, XAxis, YAxis } from "recharts";
import { type ActivityPeriod, useActivityTrend } from "@/api/time";
import { useCoreSettings } from "@/app/module-manifest";
import { axisProps, ChartLegend, ChartTooltip, gridProps, SEGMENT_GAP, tooltipProps } from "@/design";
import { ActivityPanel } from "./ActivityPanel";
import { type TrendRow, type TrendSeriesInfo, trendChart } from "./chartModels";
import { ChartTable } from "../shared/ChartTable";
import { formatDayShort } from "./dayLabel";
import { formatSpan } from "./duration";

const MAX_BAR = 24;

function TrendTooltip({ active, payload, series }: TooltipContentProps & { series: readonly TrendSeriesInfo[] }) {
  const row = payload[0]?.payload as TrendRow | undefined;
  if (!active || !row) return null;
  const title = `Week of ${formatDayShort(row.start)}${row.days < 7 ? ` (${String(row.days)} of 7 days)` : ""}`;
  return (
    <ChartTooltip
      title={title}
      rows={series.map((s) => ({ label: s.label, color: s.color, value: formatSpan(Number(row[s.key] ?? 0) * 3600) }))}
    />
  );
}

/** Screen time week by week for the busiest apps. */
export function ActivityTrend({ period }: { period: ActivityPeriod }) {
  const { week_start: weekStart } = useCoreSettings();
  const query = useActivityTrend(period, weekStart);
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
                  <YAxis {...axisProps} width={36} tickFormatter={(h: number) => `${String(h)} h`} />
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
              rows={rows.map((row) => [formatDayShort(row.start), String(row.days), ...series.map((s) => formatSpan(Number(row[s.key] ?? 0) * 3600))])}
            />
          </>
        );
      }}
    </ActivityPanel>
  );
}
