/** Local calendar dates written `YYYY-MM-DD`, stepped without time zones. Pure. */

const DAY_MS = 86_400_000;
const toMs = (iso: string) => Date.parse(`${iso}T00:00:00Z`);
const toIso = (ms: number) => new Date(ms).toISOString().slice(0, 10);

/** Every date from `from` to `to`, inclusive. */
export function eachDate(from: string, to: string): string[] {
  const dates: string[] = [];
  for (let ms = toMs(from); ms <= toMs(to); ms += DAY_MS) dates.push(toIso(ms));
  return dates;
}

/** The date `offset` days after `date` (negative for before). */
export function addDays(date: string, offset: number): string {
  return toIso(toMs(date) + offset * DAY_MS);
}

/** The Day to show a timeline for: the person's choice if it is still in range, else the last Day with Activity. */
export function pickDay(chosen: string | undefined, from: string, to: string, activeDays: readonly string[]): string | undefined {
  if (chosen !== undefined && chosen >= from && chosen <= to) return chosen;
  return activeDays.at(-1);
}
