/** The categorical chart palette: fixed hue order, never cycled (dataviz rule). */

/** How many distinct series colours exist. A ninth series folds into "Other". */
export const SERIES_COUNT = 8;

/**
 * Each series colour's value in both themes (a test keeps them equal to tokens.css), for comparing
 * an outside colour with the palette. Drawing always uses [`seriesColor`], never these.
 */
export const SERIES_HEX: readonly { light: string; dark: string }[] = [
  { light: "#2a78d6", dark: "#3987e5" },
  { light: "#eb6834", dark: "#d95926" },
  { light: "#1baf7a", dark: "#199e70" },
  { light: "#eda100", dark: "#c98500" },
  { light: "#e87ba4", dark: "#d55181" },
  { light: "#008300", dark: "#008300" },
  { light: "#6250d6", dark: "#9085e9" },
  { light: "#e34948", dark: "#e66767" },
];

/** The colour of the `index`th series (0-based) as a CSS variable. Past the eighth: the neutral "Other". */
export function seriesColor(index: number): string {
  return index >= 0 && index < SERIES_COUNT ? `var(--chart-${String(index + 1)})` : "var(--chart-other)";
}

/** The colour that stands for "Other" series. */
export const OTHER_COLOR = seriesColor(SERIES_COUNT);
