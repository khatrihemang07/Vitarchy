import { Outlet, useRouterState } from "@tanstack/react-router";
import { ModulePageFrame } from "@/design";
import { TIME_TABS, timePageFor } from "./timePages";

/** The frame every Time page shares, mounted once by the route above them, so nothing jumps and the tab pill glides between pages. */
export function TimePageLayout() {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const { title, subtitle } = timePageFor(pathname);
  return (
    <ModulePageFrame title={title} subtitle={subtitle} tabs={TIME_TABS} tabsLabel="Time pages" indicatorId="time-nav-indicator">
      <Outlet />
    </ModulePageFrame>
  );
}
