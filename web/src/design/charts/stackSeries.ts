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
}

/** A swatch that matches its bars (legend, tooltip). */
export function swatchStyle({ look, color }: Pick<StackSeries, "look" | "color">): CSSProperties {
  if (look === "solid") return { background: color };
  if (look === "hatched") return { backgroundImage: `repeating-linear-gradient(45deg, ${color} 0 2px, transparent 2px 5px)`, border: `1px solid ${color}` };
  return { background: color, border: `1px dashed ${color}` };
}
