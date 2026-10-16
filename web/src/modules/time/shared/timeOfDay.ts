/** Times of day written the way the person's Core clock setting asks. Pure. */

/** The Core clock setting. */
export type Clock = "system" | "h12" | "h24";

const hourCycle = (clock: Clock) => (clock === "h12" ? "h12" : clock === "h24" ? "h23" : undefined);

const MINUTES_PER_DAY = 1440;
/** How the end of a Day reads on every clock: "12 AM" or "00:00" would look like its start. */
const END_OF_DAY = "24:00";

/** "9:05 AM" or "09:05": minutes after local midnight (0 to 1440, where 1440 is "24:00"). */
export function formatMinuteOfDay(minute: number, clock: Clock, locale?: string): string {
  if (minute >= MINUTES_PER_DAY) return END_OF_DAY;
  const instant = new Date(Date.UTC(2000, 0, 1, 0, minute));
  return new Intl.DateTimeFormat(locale, { timeZone: "UTC", hour: "numeric", minute: "2-digit", hourCycle: hourCycle(clock) }).format(instant);
}

/** "9 AM" or "09": a whole hour of the day (0 to 24, where 24 is "24:00"). */
export function formatHourOfDay(hour: number, clock: Clock, locale?: string): string {
  if (hour >= 24) return END_OF_DAY;
  const instant = new Date(Date.UTC(2000, 0, 1, hour));
  return new Intl.DateTimeFormat(locale, { timeZone: "UTC", hour: "numeric", hourCycle: hourCycle(clock) }).format(instant);
}

/** The clock time of a UTC instant as seen in `timeZone`. */
export function formatInstant(iso: string, timeZone: string, clock: Clock, locale?: string): string {
  return new Intl.DateTimeFormat(locale, { timeZone, hour: "numeric", minute: "2-digit", hourCycle: hourCycle(clock) }).format(new Date(iso));
}
