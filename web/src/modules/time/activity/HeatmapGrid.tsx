import { useState } from "react";
import { ChartTooltip, cn } from "@/design";
import { formatHourOfDay, type Clock } from "../shared/timeOfDay";
import { formatSpan } from "./duration";
import { type HeatmapRow, intensity, maxCell } from "./heatmapModel";

const HOURS = 24;
const LABEL_EVERY = 6;
const LABEL_STARTS = Array.from({ length: HOURS / LABEL_EVERY }, (_, i) => i * LABEL_EVERY);

/** How a cell is painted: the neutral track when empty, else one hue that deepens with time. */
export function cellBackground(strength: number): string {
  return strength === 0 ? "var(--muted)" : `color-mix(in srgb, var(--chart-1) ${String(Math.round((0.15 + 0.85 * strength) * 100))}%, transparent)`;
}

/** Where a hovered cell's tooltip hangs, kept inside the grid at both ends of the day. */
const tooltipSide = (hour: number) => (hour < 6 ? "left-0" : hour >= 18 ? "right-0" : "left-1/2 -translate-x-1/2");

/**
 * The weekday-by-hour grid as a CSS grid. Cells share the width up to a maximum size; labels are
 * ordinary text, so they keep the token text size at any width. Decorative to assistive tech: the
 * table beside it carries the numbers.
 */
export function HeatmapGrid({ rows, clock }: { rows: readonly HeatmapRow[]; clock: Clock }) {
  const [hovered, setHovered] = useState<{ row: number; hour: number } | null>(null);
  const max = maxCell(rows);
  return (
    <div
      aria-hidden
      className="grid max-w-3xl gap-0.5 text-xs text-muted-foreground"
      style={{ gridTemplateColumns: `2.25rem repeat(${String(HOURS)}, minmax(0, 1fr))` }}
      onMouseLeave={() => { setHovered(null); }}
    >
      <span />
      {LABEL_STARTS.map((hour) => (
        <span key={hour} className="whitespace-nowrap" style={{ gridColumn: `span ${String(LABEL_EVERY)}` }}>
          {formatHourOfDay(hour, clock)}
        </span>
      ))}
      {rows.map((row, r) => (
        <div key={row.name} className="contents">
          <span className="flex items-center">{row.short}</span>
          {row.hours.map((seconds, hour) => {
            const open = hovered?.row === r && hovered.hour === hour;
            return (
              <span
                key={hour}
                className="relative h-5 rounded-sm"
                style={{ background: cellBackground(intensity(seconds, max)) }}
                onMouseEnter={() => { setHovered({ row: r, hour }); }}
              >
                {open ? (
                  <span className={cn("pointer-events-none absolute top-full z-10 mt-1", tooltipSide(hour))}>
                    <ChartTooltip
                      title={`${row.name}, ${formatHourOfDay(hour, clock)}`}
                      rows={[{ label: "Activity", color: "var(--chart-1)", value: formatSpan(seconds) }]}
                    />
                  </span>
                ) : null}
              </span>
            );
          })}
        </div>
      ))}
    </div>
  );
}
