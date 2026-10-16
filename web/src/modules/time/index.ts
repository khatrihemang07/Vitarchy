import { createRoute } from "@tanstack/react-router";
import { Clock } from "lucide-react";
import type { ModuleManifest } from "@/app/module-manifest";
import { TimePage } from "./TimePage";
import { parseTimeSearch } from "./timeSearch";

/** The Time Module: Toggl Time Entries and computer Activity. */
export const timeModule: ModuleManifest = {
  id: "time",
  label: "Time",
  accent: "var(--module-time)",
  icon: Clock,
  routes: (root) => [createRoute({ getParentRoute: () => root, path: "/time", component: TimePage, validateSearch: parseTimeSearch })],
  navItem: { to: "/time" },
};
