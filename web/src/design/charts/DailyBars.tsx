import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { axisLabel, fullDayLabel } from "./dayLabels";
import { niceTicks } from "./ticks";

/** One bar: a Day and the seconds on it. */
export interface DailyBarPoint {
  date: string;
  seconds: number;
}

interface DailyBarsProps {
  data: readonly DailyBarPoint[];
  /** Formats seconds for the axis, tooltip and screen readers, e.g. "1h 5m". */
  formatValue: (seconds: number) => string;
  /** Accessible name of the chart. */
  label: string;
  className?: string;
}

interface BarTooltipProps {
  active?: boolean;
  payload?: { payload: DailyBarPoint }[];
  formatValue: (seconds: number) => string;
}

function BarTooltip({ active, payload, formatValue }: BarTooltipProps) {
  const point = payload?.[0]?.payload;
  if (!active || !point) return null;
  return (
    <div className="rounded-md border border-border bg-popover px-3 py-2 text-sm text-popover-foreground shadow-md">
      <div className="text-xs text-muted-foreground">{fullDayLabel(point.date)}</div>
      <div className="font-medium tabular-nums">{formatValue(point.seconds)}</div>
    </div>
  );
}

const TICK = { fill: "var(--muted-foreground)", fontSize: 12 };

/**
 * Bars of time per Day. Colours are theme tokens, so it follows light and dark.
 * The bars are drawn for the eye; a visually hidden table gives the same
 * numbers to screen readers.
 */
export function DailyBars({ data, formatValue, label, className }: DailyBarsProps) {
  const points = [...data];
  const ticks = niceTicks(Math.max(0, ...points.map((p) => p.seconds)));
  return (
    <figure className={className} aria-label={label}>
      <div className="h-44 w-full" aria-hidden="true">
        <ResponsiveContainer width="100%" height="100%" minWidth={0}>
          <BarChart data={points} margin={{ top: 8, right: 4, bottom: 0, left: 0 }}>
            <CartesianGrid vertical={false} stroke="var(--border)" strokeDasharray="3 3" />
            <XAxis
              dataKey="date"
              tickFormatter={(day: string) => axisLabel(day, points.length)}
              tick={TICK}
              tickLine={false}
              axisLine={{ stroke: "var(--border)" }}
              interval="preserveStartEnd"
              minTickGap={8}
            />
            <YAxis
              tickFormatter={(s: number) => formatValue(s)}
              tick={TICK}
              tickLine={false}
              axisLine={false}
              width={48}
              ticks={ticks}
              domain={[0, ticks[ticks.length - 1] ?? 1]}
            />
            <Tooltip
              cursor={{ fill: "var(--accent)", opacity: 0.6 }}
              content={<BarTooltip formatValue={formatValue} />}
            />
            <Bar dataKey="seconds" fill="var(--module-accent)" radius={[4, 4, 0, 0]} isAnimationActive={false} />
          </BarChart>
        </ResponsiveContainer>
      </div>
      <table className="sr-only">
        <caption>{label}</caption>
        <tbody>
          {points.map((point) => (
            <tr key={point.date}>
              <th scope="row">{fullDayLabel(point.date)}</th>
              <td>{formatValue(point.seconds)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </figure>
  );
}
