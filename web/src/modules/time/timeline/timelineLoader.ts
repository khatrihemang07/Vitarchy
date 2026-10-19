import { coreWithDefaults, settingsOptions } from "@/api/settings";
import { timelineSummaryOptions, type TimelineQuery } from "@/api/time";
import { warmed, type WarmableQuery } from "@/app/module-manifest";
import { browserTimeZone } from "../browserTimeZone";
import { todayOf, type WeekStart } from "../reports/period";
import { rangeOf } from "../reports/reportSearch";
import { parseTimelineSearch, type TimelineSearch } from "./timelineSearch";

/** The Timeline query the page will ask for, worked out the way the page does. Pure, so both agree on one cache key. */
export function timelineQueryForLoader(search: TimelineSearch, weekStart: WeekStart, now: Date, tz: string): TimelineQuery {
  const range = rangeOf(search, todayOf(now), weekStart);
  return { kind: search.kind, from: range.from, to: range.to, tz };
}

/** Warms the Timeline page's main query from the URL's search, after the settings it waits for (defaults if they fail). */
export function warmTimeline(rawSearch: Record<string, unknown>): WarmableQuery {
  return async (queryClient) => {
    const settings = await warmed(queryClient, settingsOptions).catch(() => undefined);
    const weekStart = coreWithDefaults(settings?.core).week_start;
    const query = timelineQueryForLoader(parseTimelineSearch(rawSearch), weekStart, new Date(), browserTimeZone());
    await warmed(queryClient, timelineSummaryOptions(query));
  };
}
