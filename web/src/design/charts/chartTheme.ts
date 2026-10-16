/** Recharts props that dress every chart from design tokens, so charts follow the theme. */

/** Muted, small tick text. */
const tick = { fill: "var(--muted-foreground)", fontSize: 12 } as const;

/** Spread onto `XAxis` and `YAxis`: no heavy axis lines, muted ticks. */
export const axisProps = {
  tick,
  axisLine: { stroke: "var(--border)" },
  tickLine: false,
} as const;

/** Spread onto `CartesianGrid`: faint horizontal rules only. */
export const gridProps = {
  stroke: "var(--border)",
  strokeDasharray: "3 3",
  vertical: false,
} as const;

/** Spread onto `Tooltip`: our own tooltip card, a thin cursor and no animation jitter. */
export const tooltipProps = {
  cursor: { fill: "var(--accent)", opacity: 0.5 },
  isAnimationActive: false,
} as const;

/** Gap between stacked or adjacent fills, so segments read apart (2px surface gap). */
export const SEGMENT_GAP = 2;

/** Rounded data ends for bars, anchored to the baseline. */
export const BAR_RADIUS = 4;
