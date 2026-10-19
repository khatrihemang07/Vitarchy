import { createRoute, lazyRouteComponent } from "@tanstack/react-router";
import { Clock } from "lucide-react";
import type { ModuleManifest } from "@/app/module-manifest";
import { TIME_COMMAND_PAGES } from "./timePages";
import { TimePageLayout } from "./TimePageLayout";
import { parseReportSearch, toSearchParams } from "./reports/reportSearch";
import { parseTimelineSearch, timelineParams } from "./timeline/timelineSearch";

// Pages load on first visit; Reports and the Timeline bring the charting library with them.
const TimePage = lazyRouteComponent(() => import("./TimePage"), "TimePage");
const ReportsPage = lazyRouteComponent(() => import("./reports/ReportsPage"), "ReportsPage");
const TimelinePage = lazyRouteComponent(() => import("./timeline/TimelinePage"), "TimelinePage");

/** The Time Module: Toggl Time Entries and computer Activity. */
export const timeModule: ModuleManifest = {
  id: "time",
  label: "Time",
  accent: "var(--module-time)",
  icon: Clock,
  routes: (root) => {
    // One layout route holds the header and tabs, so they persist between the pages.
    const layout = createRoute({ getParentRoute: () => root, id: "time-layout", component: TimePageLayout });
    return [
      layout.addChildren([
        createRoute({ getParentRoute: () => layout, path: "/time", component: TimePage }),
        createRoute({
          getParentRoute: () => layout,
          path: "/time/reports",
          // The URL is user-editable: anything unusable falls back to a default. Only
          // non-default values are kept, so the address stays short and readable.
          validateSearch: (raw: Record<string, unknown>) => toSearchParams(parseReportSearch(raw)),
          component: ReportsPage,
        }),
        createRoute({
          getParentRoute: () => layout,
          path: "/time/timeline",
          validateSearch: (raw: Record<string, unknown>) => timelineParams(parseTimelineSearch(raw)),
          component: TimelinePage,
        }),
      ]),
    ];
  },
  navItem: { to: "/time" },
  pages: TIME_COMMAND_PAGES,
};
