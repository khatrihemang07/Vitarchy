import { createRoute, lazyRouteComponent } from "@tanstack/react-router";
import { Clock } from "lucide-react";
import type { ModuleManifest } from "@/app/module-manifest";
import { TimePageLayout } from "./TimePageLayout";
import { parseReportSearch, toSearchParams } from "./reports/reportSearch";

// Pages load on first visit; Reports brings the charting library with it.
const TimePage = lazyRouteComponent(() => import("./TimePage"), "TimePage");
const ReportsPage = lazyRouteComponent(() => import("./reports/ReportsPage"), "ReportsPage");

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
      ]),
    ];
  },
  navItem: { to: "/time" },
};
