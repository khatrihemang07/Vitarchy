import type { CSSProperties } from "react";

/** How a series is drawn: solid, hatched or faded, so colour is never the only cue. */
export type SeriesLook = "solid" | "hatched" | "faded";

/** One series of a per-Day bar chart: which field of a point it reads and how it looks. */
export interface StackSeries<K extends string = string> {
  /** The field of each point holding this series' value. */
  key: K;
  label: string;
  look: SeriesLook;
  /** A CSS colour, normally a design token such as `var(--module-accent)`. */
  color: string;
  /** The swatch outline of a hatched or faded series, when it differs from `color` (the fill). */
  border?: string | undefined;
}

/** A swatch that matches its bars (legend, tooltip). */
export function swatchStyle({ look, color, border = color }: Pick<StackSeries, "look" | "color" | "border">): CSSProperties {
  if (look === "solid") return { background: color };
  if (look === "hatched") return { backgroundImage: `repeating-linear-gradient(45deg, ${color} 0 2px, transparent 2px 5px)`, border: `1px solid ${border}` };
  return { background: color, border: `1px dashed ${border}` };
}
