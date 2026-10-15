/** A displayable level: whole, finite and never negative. */
export function normalizeLevel(level: number): number {
  if (!Number.isFinite(level)) return 0;
  return Math.max(0, Math.floor(level));
}
