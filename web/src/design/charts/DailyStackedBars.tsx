import { useId } from "react";
import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { axisLabel, fullDayLabel } from "./dayLabels";
import { niceTicks } from "./ticks";

/** One stacked bar: a Day and its Active, Away and unknown-Away seconds. */
export interface StackedDayPoint {
  date: string;
  active: number;
  away: number;
  unknown: number;
}

interface DailyStackedBarsProps {
  data: readonly StackedDayPoint[];
  /** Formats seconds for the axis, tooltip and screen readers, e.g. "1h 5m". */
  formatValue: (seconds: number) => string;
  /** Accessible name of the chart. */
  label: string;
  className?: string;
}

const SERIES = [
  { key: "active", label: "Active" },
  { key: "away", label: "Away" },
  { key: "unknown", label: "Away unknown" },
] as const;

type SeriesKey = (typeof SERIES)[number]["key"];

/** A swatch that matches its bars: solid, hatched or faded, so colour is never the only cue. */
function Swatch({ kind }: { kind: SeriesKey }) {
  const style =
    kind === "active"
      ? { background: "var(--module-accent)" }
      : kind === "away"
        ? { backgroundImage: "repeating-linear-gradient(45deg, var(--away) 0 2px, transparent 2px 5px)", border: "1px solid var(--away)" }
        : { background: "var(--away-unknown)", border: "1px dashed var(--away)" };
  return <span aria-hidden="true" className="inline-block size-3 rounded-sm" style={style} />;
}

interface TipProps {
  active?: boolean;
  payload?: { payload: StackedDayPoint }[];
  formatValue: (seconds: number) => string;
}

function StackTooltip({ active, payload, formatValue }: TipProps) {
  const point = payload?.[0]?.payload;
  if (!active || !point) return null;
  return (
    <div className="rounded-md border border-border bg-popover px-3 py-2 text-sm text-popover-foreground shadow-md">
      <div className="text-xs text-muted-foreground">{fullDayLabel(point.date)}</div>
      {SERIES.map(({ key, label }) => (
        <div key={key} className="flex items-center justify-between gap-4 tabular-nums">
          <span className="flex items-center gap-1.5"><Swatch kind={key} />{label}</span>
          <span className="font-medium">{formatValue(point[key])}</span>
        </div>
      ))}
    </div>
  );
}

const TICK = { fill: "var(--muted-foreground)", fontSize: 12 };
const total = (p: StackedDayPoint) => p.active + p.away + p.unknown;

/**
 * Stacked bars of Active, Away and unknown-Away time per Day, with a legend and a
 * visually hidden table for screen readers. Away is hatched and unknown is faded, so the
 * three read apart without colour. Bars are separated by a surface-coloured gap.
 */
export function DailyStackedBars({ data, formatValue, label, className }: DailyStackedBarsProps) {
  const points = [...data];
  const hatchId = `hatch-${useId().replace(/:/g, "")}`;
  const ticks = niceTicks(Math.max(0, ...points.map(total)));
  return (
    <figure className={className} aria-label={label}>
      <ul className="mb-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
        {SERIES.map(({ key, label: name }) => (
          <li key={key} className="flex items-center gap-1.5"><Swatch kind={key} />{name}</li>
        ))}
      </ul>
      <div className="h-52 w-full" aria-hidden="true">
        <ResponsiveContainer width="100%" height="100%" minWidth={0}>
          <BarChart data={points} margin={{ top: 8, right: 4, bottom: 0, left: 0 }}>
            <defs>
              <pattern id={hatchId} width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
                <rect width="6" height="6" fill="var(--card)" />
                <rect width="2" height="6" fill="var(--away)" />
              </pattern>
            </defs>
            <CartesianGrid vertical={false} stroke="var(--border)" strokeDasharray="3 3" />
            <XAxis dataKey="date" tickFormatter={(day: string) => axisLabel(day, points.length)} tick={TICK} tickLine={false} axisLine={{ stroke: "var(--border)" }} interval="preserveStartEnd" minTickGap={8} />
            <YAxis tickFormatter={(s: number) => formatValue(s)} tick={TICK} tickLine={false} axisLine={false} width={48} ticks={ticks} domain={[0, ticks[ticks.length - 1] ?? 1]} />
            <Tooltip cursor={{ fill: "var(--accent)", opacity: 0.6 }} content={<StackTooltip formatValue={formatValue} />} />
            <Bar dataKey="active" stackId="day" fill="var(--module-accent)" stroke="var(--card)" strokeWidth={1} isAnimationActive={false} />
            <Bar dataKey="away" stackId="day" fill={`url(#${hatchId})`} stroke="var(--card)" strokeWidth={1} isAnimationActive={false} />
            <Bar dataKey="unknown" stackId="day" fill="var(--away-unknown)" stroke="var(--card)" strokeWidth={1} radius={[4, 4, 0, 0]} isAnimationActive={false} />
          </BarChart>
        </ResponsiveContainer>
      </div>
      <table className="sr-only">
        <caption>{label}</caption>
        <thead>
          <tr><th scope="col">Day</th>{SERIES.map(({ key, label: name }) => <th key={key} scope="col">{name}</th>)}</tr>
        </thead>
        <tbody>
          {points.map((point) => (
            <tr key={point.date}>
              <th scope="row">{fullDayLabel(point.date)}</th>
              {SERIES.map(({ key }) => <td key={key}>{formatValue(point[key])}</td>)}
            </tr>
          ))}
        </tbody>
      </table>
    </figure>
  );
}
