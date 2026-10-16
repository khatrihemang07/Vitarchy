/** The tabs of the Time page, in display order. The id is what the URL carries. */
export const TIME_TABS = [
  { id: "overview", label: "Overview", empty: "Charts appear here once there is data for this Period." },
  { id: "activity", label: "Activity", empty: "Where your screen time goes appears here once there is Activity in this Period." },
  { id: "life-log", label: "Life log", empty: "How your days divide appears here once there are Time Entries in this Period." },
  { id: "sources", label: "Sources" },
] as const;

/** One tab's id. */
export type TimeTabId = (typeof TIME_TABS)[number]["id"];

/** The tab shown when the URL names none. */
export const DEFAULT_TAB: TimeTabId = "overview";

/** The tab ids that show a Period (Sources does not). */
export const PERIOD_TABS: readonly TimeTabId[] = ["overview", "activity", "life-log"];
