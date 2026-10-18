import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";

interface HourHeatProps {
  /** Seconds for each of the 24 hours of the day, starting at midnight. */
  values: readonly number[];
  /** The name of an hour for labels, tooltips and screen readers, e.g. "3 PM". */
  hourLabel: (hour: number) => string;
  /** Formats seconds, e.g. "1h 5m". */
  formatValue: (seconds: number) => string;
  /** Accessible name. */
  label: string;
  className?: string;
}

/** Hours that carry a printed label; labelling all 24 would not fit a phone. */
const LABEL_EVERY = 3;
/** Even the quietest hour with time in it is visibly tinted. */
const MIN_TINT = 18;

/** The fill of a cell: empty is the muted surface, otherwise the module accent from faint to full. */
function fillOf(value: number, max: number): string {
  if (value <= 0 || max <= 0) return "var(--muted)";
  const tint = MIN_TINT + (100 - MIN_TINT) * (value / max);
  return `color-mix(in srgb, var(--module-accent) ${tint.toFixed(0)}%, var(--muted))`;
}

/** 24 cells, one per hour, shaded by time. A visually hidden table carries the numbers for screen readers. */
export function HourHeat({ values, hourLabel, formatValue, label, className }: HourHeatProps) {
  const max = Math.max(0, ...values);
  return (
    <figure className={className} aria-label={label}>
      <div className="grid grid-cols-24 gap-0.5" aria-hidden="true">
        {values.map((value, hour) => (
          <Tooltip key={hour}>
            <TooltipTrigger asChild>
              <div className="h-10 rounded-sm" style={{ background: fillOf(value, max) }} />
            </TooltipTrigger>
            <TooltipContent>
              {hourLabel(hour)}: {formatValue(value)}
            </TooltipContent>
          </Tooltip>
        ))}
      </div>
      <div className="mt-1 grid grid-cols-24 gap-0.5 text-[0.7rem] text-muted-foreground tabular-nums" aria-hidden="true">
        {values.map((_, hour) => (
          <span key={hour} className="relative h-4">
            {hour % LABEL_EVERY === 0 ? <span className="absolute left-0 whitespace-nowrap">{hourLabel(hour)}</span> : null}
          </span>
        ))}
      </div>
      <table className="sr-only">
        <caption>{label}</caption>
        <tbody>
          {values.map((value, hour) => (
            <tr key={hour}>
              <th scope="row">{hourLabel(hour)}</th>
              <td>{formatValue(value)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </figure>
  );
}
