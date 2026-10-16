/** The Time page's URL search params: the tab and the Period, bookmarkable. Pure. */
import { isIsoDate, PERIOD_KINDS, selectionFor, type PeriodKind, type PeriodSelection, type WeekStart } from "./period";
import { DEFAULT_TAB, TIME_TABS, type TimeTabId } from "./tabs";

/** What the URL may carry; every part is optional. */
export interface TimeSearch {
  tab?: TimeTabId;
  period?: PeriodKind;
  from?: string;
  to?: string;
}

/** What the page shows once defaults are applied. */
export interface TimeView {
  tab: TimeTabId;
  selection: PeriodSelection;
}

const oneOf = <T extends string>(allowed: readonly T[], value: unknown): T | undefined =>
  allowed.find((candidate) => candidate === value);

const asDate = (value: unknown): string | undefined =>
  typeof value === "string" && isIsoDate(value) ? value : undefined;

/** Router `validateSearch`: keeps what is valid and drops the rest, so a bad link still opens. */
export function parseTimeSearch(raw: Record<string, unknown>): TimeSearch {
  const tab = oneOf(TIME_TABS.map((t) => t.id), raw.tab);
  const period = oneOf(PERIOD_KINDS.map((k) => k.value), raw.period);
  const from = asDate(raw.from);
  const to = period === "custom" && from ? asDate(raw.to) : undefined;
  return {
    ...(tab && { tab }),
    ...(period && { period }),
    ...(from && { from }),
    ...(to && from && to >= from && { to }),
  };
}

/** The tab and Period to show: the URL's choices, else the Overview of the current week. */
export function resolveView(search: TimeSearch, today: string, weekStart: WeekStart): TimeView {
  const kind = search.period ?? "week";
  const anchor = search.from ?? today;
  const base = selectionFor(kind, anchor, weekStart);
  const selection = kind === "custom" ? { ...base, to: search.to ?? base.to } : base;
  return { tab: search.tab ?? DEFAULT_TAB, selection };
}

/** The URL form of a view. A custom range is the only one that needs its end date. */
export function toSearch(tab: TimeTabId, selection: PeriodSelection): Required<Pick<TimeSearch, "tab" | "period" | "from">> & Pick<TimeSearch, "to"> {
  const { kind, from, to } = selection;
  return { tab, period: kind, from, ...(kind === "custom" && { to }) };
}
