const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
/** Up to this many bars there is room for weekday names under them. */
const WEEKDAY_LABELS_UP_TO = 7;

// UTC so the label never depends on the viewer's time zone: a Day is a calendar date, not an instant.
const parse = (day: string) => new Date(`${day}T00:00:00Z`);

/** "Fri 16 Oct": a Day for a tooltip or screen reader. */
export function fullDayLabel(day: string): string {
  const date = parse(day);
  return `${WEEKDAYS[date.getUTCDay()] ?? ""} ${String(date.getUTCDate())} ${MONTHS[date.getUTCMonth()] ?? ""}`;
}

/** A short axis label: the weekday for a week, otherwise the day of the month. */
export function axisLabel(day: string, barCount: number): string {
  const date = parse(day);
  return barCount <= WEEKDAY_LABELS_UP_TO ? (WEEKDAYS[date.getUTCDay()] ?? "") : String(date.getUTCDate());
}
