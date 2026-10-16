/** Durations and shares as the Activity tab writes them. Pure. */

/** "1 h 25 min", "45 min", "30 s": a length of time to the minute, without rounding a small one to zero. */
export function formatSpan(seconds: number): string {
  if (seconds < 60) return `${String(Math.max(0, Math.round(seconds)))} s`;
  const minutes = Math.round(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) return `${String(rest)} min`;
  return rest === 0 ? `${String(hours)} h` : `${String(hours)} h ${String(rest)} min`;
}

/** "42%", or "<1%" when it is not nothing but rounds to nothing. */
export function formatShare(part: number, total: number): string {
  if (total <= 0 || part <= 0) return "0%";
  const percent = (part / total) * 100;
  return percent < 1 ? "<1%" : `${String(Math.round(percent))}%`;
}

/** Hours with one decimal, "3.5": for chart axes. */
export function toHours(seconds: number): number {
  return Math.round((seconds / 3600) * 10) / 10;
}

/** How long `part` is against `longest`, 0 to 1; 0 when there is nothing to compare with. */
export function fractionOf(part: number, longest: number): number {
  return longest > 0 ? part / longest : 0;
}
