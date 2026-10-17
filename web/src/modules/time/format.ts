const MINUTE = 60;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

const dayFormat = new Intl.DateTimeFormat(undefined, { dateStyle: "medium" });
const countFormat = new Intl.NumberFormat();

/** A count such as "10,145", grouped in the person's locale. */
export function formatCount(n: number): string {
  return countFormat.format(n);
}

/** A date such as "Apr 1, 2026", in the person's locale. */
export function formatDay(day: Date | string): string {
  return dayFormat.format(new Date(day));
}

/** "just now", "5 min ago", "3 h ago", "2 d ago": how long before `now` something happened. */
export function formatAge(from: Date | string, now: Date): string {
  const secs = Math.floor((now.getTime() - new Date(from).getTime()) / 1000);
  if (secs < 45) return "just now";
  if (secs < HOUR) return `${String(Math.max(1, Math.round(secs / MINUTE)))} min ago`;
  if (secs < DAY) return `${String(Math.round(secs / HOUR))} h ago`;
  return `${String(Math.round(secs / DAY))} d ago`;
}

/** "40 s", "5 min", "2 h": a length of time, rounded up so a wait is never understated. */
export function formatDuration(secs: number): string {
  if (secs < MINUTE) return `${String(Math.max(1, Math.ceil(secs)))} s`;
  if (secs < HOUR) return `${String(Math.ceil(secs / MINUTE))} min`;
  return `${String(Math.ceil(secs / HOUR))} h`;
}

/** "1h 5m", "25m", "40s": how long something took, to the minute (seconds only under a minute). Sign is ignored. */
export function formatSpan(secs: number): string {
  const abs = Math.abs(secs);
  if (abs > 0 && abs < MINUTE) return `${String(Math.round(abs))}s`;
  const minutes = Math.round(abs / MINUTE);
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  if (h === 0) return `${String(m)}m`;
  return m === 0 ? `${String(h)}h` : `${String(h)}h ${String(m)}m`;
}
