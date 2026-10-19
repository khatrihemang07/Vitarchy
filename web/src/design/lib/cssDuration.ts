/** A CSS time such as `250ms` or `0.25s` in milliseconds; anything else (including empty) is 0. */
export function parseCssDuration(raw: string): number {
  const match = /^\s*(\d*\.?\d+)\s*(ms|s)\s*$/.exec(raw);
  if (!match) return 0;
  const value = Number(match[1]);
  return match[2] === "s" ? value * 1000 : value;
}
