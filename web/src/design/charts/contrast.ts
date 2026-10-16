/** Whether an outside colour (e.g. one Toggl chose) is safe to draw a chart series in. Pure. */

/** The card colour charts sit on in the light and dark themes; a test keeps these equal to `--card` in tokens.css. */
export const CHART_SURFACES = ["#ffffff", "#17171f"] as const;

/** The least contrast a mark needs against its surface (WCAG non-text contrast). */
export const MIN_MARK_CONTRAST = 3;

/** The least RGB distance that keeps two series apart at a glance. */
const MIN_DISTANCE = 40;

const HEX = /^#[0-9a-f]{6}$/i;

const channels = (hex: string): [number, number, number] => [1, 3, 5].map((i) => Number.parseInt(hex.slice(i, i + 2), 16)) as [number, number, number];

function luminance(hex: string): number {
  const [r, g, b] = channels(hex).map((value) => {
    const c = value / 255;
    return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  }) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG contrast ratio of two `#rrggbb` colours, 1 to 21. */
export function contrastRatio(a: string, b: string): number {
  const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (light + 0.05) / (dark + 0.05);
}

function distance(a: string, b: string): number {
  const [ca, cb] = [channels(a), channels(b)];
  return Math.hypot(ca[0] - cb[0], ca[1] - cb[1], ca[2] - cb[2]);
}

/** Whether `color` (`#rrggbb`) is far enough from every colour in `taken` to tell apart at a glance. */
export function isDistinct(color: string, taken: readonly string[]): boolean {
  return taken.every((other) => distance(color, other) >= MIN_DISTANCE);
}

/**
 * `color` when it is a `#rrggbb` that shows up on both themes' surfaces and is not close to a colour
 * in `taken`; otherwise `undefined`, and the caller falls back to the palette.
 */
export function usableSeriesColor(color: string | null | undefined, taken: readonly string[]): string | undefined {
  if (!color || !HEX.test(color)) return undefined;
  if (CHART_SURFACES.some((surface) => contrastRatio(color, surface) < MIN_MARK_CONTRAST)) return undefined;
  if (!isDistinct(color, taken)) return undefined;
  return color;
}
