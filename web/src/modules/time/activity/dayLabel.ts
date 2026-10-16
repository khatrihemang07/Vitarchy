const short = new Intl.DateTimeFormat(undefined, { timeZone: "UTC", month: "short", day: "numeric" });
const long = new Intl.DateTimeFormat(undefined, { timeZone: "UTC", weekday: "short", month: "short", day: "numeric" });
const at = (date: string) => new Date(`${date}T00:00:00Z`);

/** "May 4": a local date `YYYY-MM-DD` on an axis. */
export function formatDayShort(date: string): string {
  return short.format(at(date));
}

/** "Mon, May 4": a local date `YYYY-MM-DD` with its weekday. */
export function formatDayLong(date: string): string {
  return long.format(at(date));
}
