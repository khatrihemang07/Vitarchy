import type { ReportEntry, ReportGroup } from "@/api/time";

/** The Core clock Setting. */
export type Clock = "system" | "h12" | "h24";

const CLOCK_OPTIONS: Record<Clock, Intl.DateTimeFormatOptions> = {
  system: { timeStyle: "short" },
  h12: { hour: "numeric", minute: "2-digit", hour12: true },
  h24: { hour: "2-digit", minute: "2-digit", hourCycle: "h23" },
};

/** e.g. "6:05 PM" or "18:05" per the Core clock Setting (a plain space, not Intl's narrow no-break one). */
export function formatClock(date: Date, clock: Clock): string {
  const locale = clock === "system" ? undefined : "en-US";
  return new Intl.DateTimeFormat(locale, CLOCK_OPTIONS[clock]).format(date).replace(/[\u202f\u00a0]/g, " ");
}

const dateFormat = new Intl.DateTimeFormat("en-GB", { weekday: "short", day: "numeric", month: "short" });

/** How one entry reads in the list. */
export interface EntryTimes {
  /** The Day it started, e.g. "Fri 16 Oct". */
  date: string;
  start: string;
  stop: string;
  /** True when it stops on a later Day than it started. */
  crossesMidnight: boolean;
}

const sameDay = (a: Date, b: Date) =>
  a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();

/** Start, stop and Day of an entry in the browser's time zone, which is the Report's. */
export function describeEntry(entry: ReportEntry, clock: Clock): EntryTimes {
  const start = new Date(entry.start);
  const stop = new Date(entry.stop);
  return {
    date: dateFormat.format(start).replace(",", ""),
    start: formatClock(start, clock),
    stop: formatClock(stop, clock),
    crossesMidnight: !sameDay(start, stop),
  };
}

type EntryHolder = Pick<ReportGroup, "entries"> & { children: EntryHolder[] };

function collect(groups: EntryHolder[], into: Map<number, ReportEntry>): void {
  for (const group of groups) {
    for (const entry of group.entries) into.set(entry.toggl_id, entry);
    collect(group.children, into);
  }
}

/** Every entry in the bottom-level groups once (a two-tag entry sits in two groups), earliest first. */
export function chronologicalEntries(groups: EntryHolder[]): ReportEntry[] {
  const found = new Map<number, ReportEntry>();
  collect(groups, found);
  return [...found.values()].sort((a, b) => Date.parse(a.start) - Date.parse(b.start));
}
