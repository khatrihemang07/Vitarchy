import type { PeriodKind } from "@/api/time";
import type { DayRange } from "../reports/period";
import { withRange } from "../reports/reportActions";
import { parsePeriodSearch, type PeriodSearch, periodParams } from "../reports/reportSearch";

/** Everything the URL of the Timeline page says: the Period (as on Reports) and the highlighted app. */
export interface TimelineSearch extends PeriodSearch {
  /** The app whose Segments are highlighted; absent for none. */
  app?: string;
}

const MAX_APP_LENGTH = 200;

/** Turns whatever the URL held into a valid state; the URL is user-editable, so nothing is trusted. */
export function parseTimelineSearch(raw: Record<string, unknown>): TimelineSearch {
  const app = typeof raw.app === "string" || typeof raw.app === "number" ? String(raw.app).slice(0, MAX_APP_LENGTH) : "";
  return { ...parsePeriodSearch(raw), ...(app && { app }) };
}

/** The URL form of a state: defaults are left out. */
export function timelineParams(search: TimelineSearch): Record<string, unknown> {
  return { ...periodParams(search), ...(search.app && { app: search.app }) };
}

/** The state after moving to another Period; the highlight is dropped because the apps differ. */
export function withPeriod(search: TimelineSearch, kind: PeriodKind, range: DayRange): TimelineSearch {
  const next: TimelineSearch = { ...search };
  delete next.app;
  return withRange(next, kind, range);
}

/** The state with `app` highlighted; choosing the highlighted app again (or none) removes it. */
export function withApp(search: TimelineSearch, app: string | undefined): TimelineSearch {
  const { app: current, ...rest } = search;
  return app && app !== current ? { ...rest, app } : rest;
}
