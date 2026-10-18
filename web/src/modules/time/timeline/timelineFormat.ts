import type { Away } from "@/api/time";
import { type Clock, formatClock, formatDate } from "../reports/entryTimes";

/** "15" or "3 PM": an hour mark short enough for a narrow band. */
export function formatHour(date: Date, clock: Clock): string {
  const h12 = clock === "h12";
  const options: Intl.DateTimeFormatOptions = h12 ? { hour: "numeric", hour12: true } : { hour: "2-digit", hourCycle: "h23" };
  const locale = clock === "system" ? undefined : "en-US";
  const text = new Intl.DateTimeFormat(locale, clock === "system" ? { hour: "numeric" } : options).format(date);
  return text.replace(/\s/g, " ");
}

/** "43%", "<1%" for a tiny share, so a real figure never reads as zero. */
export function formatShare(share: number): string {
  if (share > 0 && share < 0.005) return "<1%";
  return `${String(Math.round(share * 100))}%`;
}

/** A rate such as Switches per Active hour, to one decimal; a dash when there is none. */
export function formatPerHour(rate: number | null | undefined): string {
  return rate == null ? "—" : rate.toFixed(1);
}

const AWAY_WORDS: Record<Away, string> = { no: "Active", yes: "Away", unknown: "Away unknown" };

/** A Segment's state in words. */
export function describeAway(away: Away): string {
  return AWAY_WORDS[away];
}

/** "15:05", or "Fri 16 Oct, 15:05" when `withDate`; a dash for a missing moment. */
export function formatMoment(iso: string | null | undefined, clock: Clock, withDate: boolean): string {
  if (!iso) return "—";
  const date = new Date(iso);
  return withDate ? `${formatDate(date)}, ${formatClock(date, clock)}` : formatClock(date, clock);
}
