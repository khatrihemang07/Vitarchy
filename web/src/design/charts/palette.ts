/** The categorical chart palette: fixed hue order, never cycled (dataviz rule). */

/** How many distinct series colours exist. A ninth series folds into "Other". */
export const SERIES_COUNT = 8;

/** The colour of the `index`th series (0-based) as a CSS variable. Past the eighth: the neutral "Other". */
export function seriesColor(index: number): string {
  return index >= 0 && index < SERIES_COUNT ? `var(--chart-${String(index + 1)})` : "var(--chart-other)";
}

/** The colour that stands for "Other" series. */
export const OTHER_COLOR = seriesColor(SERIES_COUNT);
