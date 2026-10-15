import type { CoreSettings } from "@/api/settings";

type Locale = string | undefined;

const pad = (n: number) => String(n).padStart(2, "0");

/** `Intl` may put a no-break space before AM/PM; a plain space is friendlier. */
const withPlainSpaces = (text: string) => text.replace(/[\u202f\u00a0]/g, " ");

const englishDatePart = (date: Date, options: Intl.DateTimeFormatOptions) =>
  new Intl.DateTimeFormat("en-US", options).format(date);

type DateFormatter = (date: Date, locale: Locale) => string;

const DATE_FORMATTERS: Record<CoreSettings["date_format"], DateFormatter> = {
  system: (date, locale) => new Intl.DateTimeFormat(locale, { dateStyle: "full" }).format(date),
  day_month_year: (date) =>
    `${englishDatePart(date, { weekday: "long" })}, ${String(date.getDate())} ${englishDatePart(date, { month: "long" })} ${String(date.getFullYear())}`,
  month_day_year: (date) =>
    `${englishDatePart(date, { weekday: "long" })}, ${englishDatePart(date, { month: "long" })} ${String(date.getDate())}, ${String(date.getFullYear())}`,
  iso: (date) => `${String(date.getFullYear())}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`,
};

/** e.g. "Friday, October 16, 2026". `locale` only matters for the System format. */
export function formatDate(
  date: Date,
  settings: Pick<CoreSettings, "date_format">,
  locale?: Locale,
): string {
  return DATE_FORMATTERS[settings.date_format](date, locale);
}

const CLOCK_OPTIONS: Record<CoreSettings["clock"], Intl.DateTimeFormatOptions> = {
  system: { timeStyle: "short" },
  h12: { hour: "numeric", minute: "2-digit", hour12: true },
  h24: { hour: "2-digit", minute: "2-digit", hourCycle: "h23" },
};

/** e.g. "6:05 PM" or "18:05". `locale` only matters for the System clock. */
export function formatTime(date: Date, settings: Pick<CoreSettings, "clock">, locale?: Locale): string {
  const resolved = settings.clock === "system" ? locale : "en-US";
  return withPlainSpaces(new Intl.DateTimeFormat(resolved, CLOCK_OPTIONS[settings.clock]).format(date));
}

/** A fixed day (Friday) that shows each format's day, month and year distinctly. */
const SAMPLE_DATE = new Date(2026, 9, 16);

/**
 * An option label for a date format: the real formatter applied to a sample
 * date, so the label always matches what the format produces.
 */
export function dateFormatLabel(format: CoreSettings["date_format"], locale?: Locale): string {
  const sample = formatDate(SAMPLE_DATE, { date_format: format }, locale);
  return format === "system" ? `System (${sample})` : sample;
}
