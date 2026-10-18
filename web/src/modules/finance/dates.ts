const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"] as const;

const DAY = /^(\d{4})-(\d{2})-(\d{2})/;

/** "1 Apr 2026" from a `YYYY-MM-DD` date (no time zone shift: a printed date is just a date). */
export function formatDate(day: string): string {
  const match = DAY.exec(day);
  const month = match ? MONTHS[Number(match[2]) - 1] : undefined;
  if (!match?.[1] || !match[3] || !month) return day;
  return `${String(Number(match[3]))} ${month} ${match[1]}`;
}

/** "1 Apr 2026 to 30 Apr 2026", one date when both ends match, or what is known when one end is missing. */
export function formatPeriod(period: { start?: string | null | undefined; end?: string | null | undefined }): string {
  const { start, end } = period;
  if (start && end) return start === end ? formatDate(start) : `${formatDate(start)} to ${formatDate(end)}`;
  if (start) return `from ${formatDate(start)}`;
  if (end) return `to ${formatDate(end)}`;
  return "Period not printed";
}

/** "1 Apr 2026, 10:15" from a timestamp without a time zone, as a Payment record carries it. */
export function formatDateTime(stamp: string): string {
  const time = /T(\d{2}:\d{2})/.exec(stamp)?.[1];
  return time ? `${formatDate(stamp)}, ${time}` : formatDate(stamp);
}

/** "Last used 3 May 2026" or "Never used". */
export function formatLastUsed(stamp: string | null | undefined): string {
  return stamp ? `Last used ${formatDate(stamp.slice(0, 10))}` : "Never used";
}
