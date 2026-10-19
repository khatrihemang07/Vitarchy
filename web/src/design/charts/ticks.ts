/** Step sizes in seconds that read well on a time axis: minutes, then hours. */
const STEPS = [60, 120, 300, 600, 900, 1800, 3600, 7200, 10_800, 14_400, 21_600, 28_800, 43_200, 86_400];
const MAX_STEPS = 4;

/**
 * Axis ticks (seconds) from 0 up to just above `max`, spaced by a round amount
 * (e.g. every 3h) so the labels read as clean times, unlike an automatic scale.
 */
export function niceTicks(max: number): number[] {
  const step = STEPS.find((s) => Math.ceil(max / s) <= MAX_STEPS) ?? STEPS[STEPS.length - 1] ?? 86_400;
  const count = Math.max(1, Math.ceil(max / step));
  return Array.from({ length: count + 1 }, (_, i) => i * step);
}

/**
 * Axis ticks from 0 up to just above `max` for plain quantities such as money: a 1, 2 or 5
 * times a power of ten apart, at most four steps, so the axis starts at 0 and reads round.
 */
export function decimalTicks(max: number): number[] {
  if (!(max > 0)) return [0, 1];
  const magnitude = 10 ** Math.floor(Math.log10(max / MAX_STEPS));
  const step = [1, 2, 5, 10].map((m) => m * magnitude).find((s) => Math.ceil(max / s) <= MAX_STEPS) ?? magnitude * 10;
  const count = Math.ceil(max / step);
  return Array.from({ length: count + 1 }, (_, i) => i * step);
}
