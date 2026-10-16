import { formatCount } from "../format";

/** Entries and days under a name, for narrow screens where those columns are hidden. */
export function CountsLine({ entries, days }: { entries: number; days: number }) {
  return (
    <span className="block truncate text-xs font-normal text-muted-foreground sm:hidden">
      {formatCount(entries)} {entries === 1 ? "entry" : "entries"} · {formatCount(days)} {days === 1 ? "day" : "days"}
    </span>
  );
}
