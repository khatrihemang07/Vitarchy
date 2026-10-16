/** Times of day written the way the person's Core clock setting asks. Pure. */

/** The Core clock setting. */
export type Clock = "system" | "h12" | "h24";

const hourCycle = (clock: Clock) => (clock === "h12" ? "h12" : clock === "h24" ? "h23" : undefined);

/** "9:05 AM" or "09:05": minutes after local midnight (0 to 1440). */
export function formatMinuteOfDay(minute: number, clock: Clock, locale?: string): string {
  const instant = new Date(Date.UTC(2000, 0, 1, 0, minute));
  return new Intl.DateTimeFormat(locale, { timeZone: "UTC", hour: "numeric", minute: "2-digit", hourCycle: hourCycle(clock) }).format(instant);
}

/** "9 AM" or "09": a whole hour of the day (0 to 24). */
export function formatHourOfDay(hour: number, clock: Clock, locale?: string): string {
  const instant = new Date(Date.UTC(2000, 0, 1, hour));
  return new Intl.DateTimeFormat(locale, { timeZone: "UTC", hour: "numeric", hourCycle: hourCycle(clock) }).format(instant);
}

/** The clock time of a UTC instant as seen in `timeZone`. */
export function formatInstant(iso: string, timeZone: string, clock: Clock, locale?: string): string {
  return new Intl.DateTimeFormat(locale, { timeZone, hour: "numeric", minute: "2-digit", hourCycle: hourCycle(clock) }).format(new Date(iso));
}
