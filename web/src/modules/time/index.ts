import { createRoute } from "@tanstack/react-router";
import { Clock } from "lucide-react";
import type { ModuleManifest } from "@/app/module-manifest";
import { ReportsPage } from "./reports/ReportsPage";
import { parseReportSearch, toSearchParams } from "./reports/reportSearch";
import { TimePage } from "./TimePage";

/** The Time Module: Toggl Time Entries and computer Activity. */
export const timeModule: ModuleManifest = {
  id: "time",
  label: "Time",
  accent: "var(--module-time)",
  icon: Clock,
  routes: (root) => [
    createRoute({ getParentRoute: () => root, path: "/time", component: TimePage }),
    createRoute({
      getParentRoute: () => root,
      path: "/time/reports",
      // The URL is user-editable: anything unusable falls back to a default. Only
      // non-default values are kept, so the address stays short and readable.
      validateSearch: (raw: Record<string, unknown>) => toSearchParams(parseReportSearch(raw)),
      component: ReportsPage,
    }),
  ],
  navItem: { to: "/time" },
};
