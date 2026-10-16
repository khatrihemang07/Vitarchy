import { useState } from "react";
import { ChartTooltip } from "@/design";
import { formatHourOfDay, type Clock } from "../shared/timeOfDay";
import { formatSpan } from "./duration";
import { type HeatmapRow, intensity, maxCell } from "./heatmapModel";

const CELL = 14;
const GAP = 2;
const LABEL_W = 34;
const TOP = 16;
const STEP = CELL + GAP;
const HOURS = 24;
const WIDTH = LABEL_W + HOURS * STEP;
const HEIGHT_OF = (rows: number) => TOP + rows * STEP;
const LABEL_EVERY = 3;

/** How a cell is filled: the neutral track when empty, else one hue that deepens with time. */
export function cellFill(strength: number): { fill: string; fillOpacity: number } {
  return strength === 0 ? { fill: "var(--muted)", fillOpacity: 1 } : { fill: "var(--chart-1)", fillOpacity: 0.15 + 0.85 * strength };
}

/** The weekday-by-hour grid as an SVG that scales to its container. Decorative to assistive tech: the table beside it carries the numbers. */
export function HeatmapGrid({ rows, clock }: { rows: readonly HeatmapRow[]; clock: Clock }) {
  const [hovered, setHovered] = useState<{ row: number; hour: number } | null>(null);
  const max = maxCell(rows);
  const active = hovered ? rows[hovered.row] : undefined;
  return (
    <div className="relative">
      <svg
        viewBox={`0 0 ${String(WIDTH)} ${String(HEIGHT_OF(rows.length))}`}
        className="block h-auto w-full"
        aria-hidden
        onMouseLeave={() => { setHovered(null); }}
      >
        {Array.from({ length: HOURS / LABEL_EVERY }, (_, i) => i * LABEL_EVERY).map((hour) => (
          <text key={hour} x={LABEL_W + hour * STEP} y={10} fontSize={10} fill="var(--muted-foreground)">
            {formatHourOfDay(hour, clock)}
          </text>
        ))}
        {rows.map((row, r) => (
          <g key={row.name}>
            <text x={0} y={TOP + r * STEP + CELL - 3} fontSize={11} fill="var(--muted-foreground)">
              {row.short}
            </text>
            {row.hours.map((seconds, hour) => (
              <rect
                key={hour}
                x={LABEL_W + hour * STEP}
                y={TOP + r * STEP}
                width={CELL}
                height={CELL}
                rx={3}
                {...cellFill(intensity(seconds, max))}
                onMouseEnter={() => { setHovered({ row: r, hour }); }}
              />
            ))}
          </g>
        ))}
      </svg>
      {hovered && active ? (
        <div
          className="pointer-events-none absolute z-10 -translate-x-1/2"
          style={{
            left: `${String(((LABEL_W + hovered.hour * STEP + CELL / 2) / WIDTH) * 100)}%`,
            top: `${String(((TOP + hovered.row * STEP + CELL) / HEIGHT_OF(rows.length)) * 100)}%`,
          }}
        >
          <ChartTooltip
            title={`${active.name}, ${formatHourOfDay(hovered.hour, clock)}`}
            rows={[{ label: "Activity", color: "var(--chart-1)", value: formatSpan(active.hours[hovered.hour] ?? 0) }]}
          />
        </div>
      ) : null}
    </div>
  );
}
