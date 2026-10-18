/** How many apps get a colour of their own; the rest share Other. */
export const PALETTE_SIZE = 8;

/** The colour of every app past the palette. */
export const OTHER_COLOR = "var(--chart-other)";

/** App name to chart colour. */
export type AppColors = ReadonlyMap<string, string>;

/**
 * Colours for a response's apps: rank 1 to 8 take `--chart-1…8`, the rest are Other. The
 * band, list and legend all take this one map, so an app is one colour everywhere on the page.
 */
export function appColorMap(apps: readonly { app: string }[]): AppColors {
  return new Map(apps.slice(0, PALETTE_SIZE).map(({ app }, rank) => [app, `var(--chart-${String(rank + 1)})`]));
}

/** An app's colour; apps outside the palette (or not in the response) are Other. */
export function colorOf(colors: AppColors, app: string): string {
  return colors.get(app) ?? OTHER_COLOR;
}
