import { useId } from "react";
import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { axisLabel, fullDayLabel } from "./dayLabels";
import { type StackSeries, swatchStyle } from "./stackSeries";
import { niceTicks } from "./ticks";

/** One Day's values: the date and a number for each series' key. */
export type StackedDayPoint<K extends string = string> = { date: string } & Record<K, number>;

interface DailyStackedBarsProps<K extends string> {
  data: readonly StackedDayPoint<K>[];
  series: readonly StackSeries<K>[];
  /** Stack the series on one bar (parts of a whole) or set them side by side (to compare). */
  stacked?: boolean;
  /** Formats a value for the axis, tooltip and screen readers, e.g. "1h 5m". */
  formatValue: (value: number) => string;
  /** A shorter form for the axis (e.g. "₹1.2L"); defaults to `formatValue`. */
  formatAxis?: (value: number) => string;
  /** Where the axis ticks fall, from 0 to above the largest bar; defaults to round times. */
  ticksFor?: (max: number) => number[];
  /** Width in pixels of the value axis; widen it for longer labels such as money. */
  axisWidth?: number;
  /** Accessible name of the chart. */
  label: string;
  className?: string;
}

function Swatch(props: Pick<StackSeries, "look" | "color" | "border">) {
  return <span aria-hidden="true" className="inline-block size-3 rounded-sm" style={swatchStyle(props)} />;
}

interface TipProps<K extends string> {
  active?: boolean;
  payload?: { payload: StackedDayPoint<K> }[];
  series: readonly StackSeries<K>[];
  formatValue: (value: number) => string;
}

function StackTooltip<K extends string>({ active, payload, series, formatValue }: TipProps<K>) {
  const point = payload?.[0]?.payload;
  if (!active || !point) return null;
  return (
    <div className="rounded-md border border-border bg-popover px-3 py-2 text-sm text-popover-foreground shadow-md">
      <div className="text-xs text-muted-foreground">{fullDayLabel(point.date)}</div>
      {series.map((s) => (
        <div key={s.key} className="flex items-center justify-between gap-4 tabular-nums">
          <span className="flex items-center gap-1.5"><Swatch look={s.look} color={s.color} border={s.border} />{s.label}</span>
          <span className="font-medium">{formatValue(point[s.key])}</span>
        </div>
      ))}
    </div>
  );
}

const TICK = { fill: "var(--muted-foreground)", fontSize: 12 };

/** What a series' bars are filled with: its colour, or the hatch pattern for a hatched one. */
const fillOf = (s: StackSeries, hatchId: string) => (s.look === "hatched" ? `url(#${hatchId}-${s.key})` : s.color);

/**
 * Per-Day bars of one or more series, with a legend and a visually hidden table for screen
 * readers. Series differ in look (solid, hatched, faded), so they read apart without colour.
 * Stacked bars are separated by a surface-coloured gap.
 */
export function DailyStackedBars<K extends string>({ data, series, stacked = true, formatValue, formatAxis = formatValue, ticksFor = niceTicks, axisWidth = 48, label, className }: DailyStackedBarsProps<K>) {
  const points = [...data];
  const hatchId = `hatch-${useId().replace(/:/g, "")}`;
  const dayMax = (p: StackedDayPoint<K>) => (stacked ? series.reduce((sum, s) => sum + p[s.key], 0) : Math.max(...series.map((s) => p[s.key])));
  const ticks = ticksFor(Math.max(0, ...points.map(dayMax)));
  return (
    <figure className={className} aria-label={label}>
      <ul className="mb-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
        {series.map((s) => (
          <li key={s.key} className="flex items-center gap-1.5"><Swatch look={s.look} color={s.color} border={s.border} />{s.label}</li>
        ))}
      </ul>
      <div className="h-52 w-full" aria-hidden="true">
        <ResponsiveContainer width="100%" height="100%" minWidth={0}>
          <BarChart data={points} margin={{ top: 8, right: 4, bottom: 0, left: 0 }}>
            <defs>
              {series.filter((s) => s.look === "hatched").map((s) => (
                <pattern key={s.key} id={`${hatchId}-${s.key}`} width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
                  <rect width="6" height="6" fill="var(--card)" />
                  <rect width="2" height="6" fill={s.color} />
                </pattern>
              ))}
            </defs>
            <CartesianGrid vertical={false} stroke="var(--border)" strokeDasharray="3 3" />
            <XAxis dataKey="date" tickFormatter={(day: string) => axisLabel(day, points.length)} tick={TICK} tickLine={false} axisLine={{ stroke: "var(--border)" }} interval="preserveStartEnd" minTickGap={8} />
            <YAxis tickFormatter={(v: number) => formatAxis(v)} tick={TICK} tickLine={false} axisLine={false} width={axisWidth} ticks={ticks} domain={[0, ticks[ticks.length - 1] ?? 1]} />
            <Tooltip cursor={{ fill: "var(--accent)", opacity: 0.6 }} content={<StackTooltip series={series} formatValue={formatValue} />} />
            {series.map((s, index) => (
              <Bar key={s.key} dataKey={s.key} {...(stacked && { stackId: "day" })} fill={fillOf(s, hatchId)} stroke={stacked ? "var(--card)" : s.color} strokeWidth={1} radius={!stacked || index === series.length - 1 ? [4, 4, 0, 0] : 0} isAnimationActive={false} />
            ))}
          </BarChart>
        </ResponsiveContainer>
      </div>
      <table className="sr-only">
        <caption>{label}</caption>
        <thead>
          <tr><th scope="col">Day</th>{series.map((s) => <th key={s.key} scope="col">{s.label}</th>)}</tr>
        </thead>
        <tbody>
          {points.map((point) => (
            <tr key={point.date}>
              <th scope="row">{fullDayLabel(point.date)}</th>
              {series.map((s) => <td key={s.key}>{formatValue(point[s.key])}</td>)}
            </tr>
          ))}
        </tbody>
      </table>
    </figure>
  );
}
