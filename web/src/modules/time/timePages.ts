import { USUAL_SPAN } from "./usualWords";

/** Each Time page keyed by its path: its tab label and heading, so the tabs and the shared frame derive from one place. */
export const TIME_PAGES = {
  "/time": { tab: "Sources", title: "Time", subtitle: "Your Toggl Time Entries and computer Activity." },
  "/time/reports": { tab: "Reports", title: "Reports", subtitle: `Where your Time Entries went, compared with the Previous Period or the ${USUAL_SPAN}.` },
} as const;

type TimePagePath = keyof typeof TIME_PAGES;

/** The heading for a pathname; anything unknown (a trailing slash, a future page) falls back to the Sources page. */
export function timePageFor(pathname: string): (typeof TIME_PAGES)[TimePagePath] {
  const path = pathname.replace(/\/+$/, "") || "/";
  return path in TIME_PAGES ? TIME_PAGES[path as TimePagePath] : TIME_PAGES["/time"];
}

/** The tabs of the Time pages, in page order. */
export const TIME_TABS = (Object.keys(TIME_PAGES) as TimePagePath[]).map((to) => ({ to, label: TIME_PAGES[to].tab }));

/** The Time pages as the command bar lists them ("Time: Reports"), derived from the same table. */
export const TIME_COMMAND_PAGES = (Object.keys(TIME_PAGES) as TimePagePath[]).map((to) => ({
  id: TIME_PAGES[to].tab.toLowerCase(),
  label: TIME_PAGES[to].tab,
  to,
}));
