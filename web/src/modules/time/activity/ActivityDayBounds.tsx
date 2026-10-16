import { CartesianGrid, Line, LineChart, ResponsiveContainer, Tooltip, type TooltipContentProps, XAxis, YAxis } from "recharts";
import type { ActivityPeriod } from "@/api/time";
import { useCoreSettings } from "@/app/module-manifest";
import { axisProps, ChartLegend, gridProps, seriesColor, tooltipProps } from "@/design";
import { ChartPanel } from "../shared/ChartPanel";
import { BoundsTooltip } from "./BoundsTooltip";
import { boundsPoints } from "./chartModels";
import { ChartTable } from "../shared/ChartTable";
import { formatDayShort } from "../shared/dayLabel";
import { formatMinuteOfDay } from "../shared/timeOfDay";
import { useActivityPart } from "./useActivityPart";

const FIRST = seriesColor(0);
const LAST = seriesColor(1);
const LINES = [
  { key: "first", color: FIRST },
  { key: "last", color: LAST },
] as const;
const TICKS = [0, 360, 720, 1080, 1440];
const DOTS_UNTIL = 31;

/** When each Day's Activity began and ended, read down the page from midnight. */
export function ActivityDayBounds({ period }: { period: ActivityPeriod }) {
  const query = useActivityPart(period, (overview) => overview.bounds);
  const { clock } = useCoreSettings();
  return (
    <ChartPanel
      title="First and last activity of each day"
      description="When the computer was first and last used, by time of day."
      query={query}
      isEmpty={(data) => data.length === 0}
      emptyMessage="No Activity in this Period."
    >
      {(data) => {
        const points = boundsPoints(period.from, period.to, data);
        return (
          <>
            <ChartLegend
              items={[
                { label: "First activity", color: FIRST },
                { label: "Last activity", color: LAST },
              ]}
            />
            <div role="img" aria-label="Line chart of the first and last Activity of each Day. The same numbers follow as a table." className="h-64 w-full min-w-0">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={points} margin={{ top: 16, right: 12, bottom: 0, left: 0 }}>
                  <CartesianGrid {...gridProps} />
                  <XAxis dataKey="date" {...axisProps} tickFormatter={formatDayShort} interval="preserveStartEnd" minTickGap={32} />
                  <YAxis {...axisProps} reversed domain={[0, 1440]} ticks={TICKS} width={64} tickFormatter={(m: number) => formatMinuteOfDay(m, clock)} />
                  <Tooltip {...tooltipProps} content={(props: TooltipContentProps) => <BoundsTooltip {...props} clock={clock} first={FIRST} last={LAST} />} />
                  {LINES.map(({ key, color }) => (
                    <Line
                      key={key}
                      dataKey={key}
                      stroke={color}
                      strokeWidth={2}
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      dot={points.length <= DOTS_UNTIL ? { r: 4, fill: color, stroke: "var(--card)", strokeWidth: 2 } : false}
                      activeDot={{ r: 5, fill: color, stroke: "var(--card)", strokeWidth: 2 }}
                      isAnimationActive={false}
                    />
                  ))}
                </LineChart>
              </ResponsiveContainer>
            </div>
            <ChartTable
              caption="First and last Activity of each Day"
              columns={["Day", "First", "Last"]}
              rows={data.map((day) => [
                formatDayShort(day.date),
                formatMinuteOfDay(day.first_minute, clock),
                formatMinuteOfDay(day.last_minute, clock),
              ])}
            />
          </>
        );
      }}
    </ChartPanel>
  );
}
