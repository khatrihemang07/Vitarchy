/** Progress inside the current level. */
export interface XpProgress {
  /** Value clamped into [0, max]. */
  value: number;
  max: number;
  /** Whole-number percentage, 0 to 100. */
  percent: number;
}

/** Pure progress math behind XpBar: clamps and rounds, never throws. */
export function xpProgress(value: number, max: number): XpProgress {
  const safeMax = Number.isFinite(max) && max > 0 ? max : 0;
  const safeValue = Number.isFinite(value) ? Math.min(Math.max(value, 0), safeMax) : 0;
  const percent = safeMax === 0 ? 0 : Math.round((safeValue / safeMax) * 100);
  return { value: safeValue, max: safeMax, percent };
}
