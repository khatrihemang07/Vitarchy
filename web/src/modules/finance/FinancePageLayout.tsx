import { Outlet, useRouterState } from "@tanstack/react-router";
import { ModulePageFrame } from "@/design";
import { FINANCE_TABS, financePageFor } from "./financePages";

/** The frame every Finance page shares, mounted once by the route above them: header, tabs and the page. */
export function FinancePageLayout() {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const { title, subtitle } = financePageFor(pathname);
  return (
    <ModulePageFrame title={title} subtitle={subtitle} tabs={FINANCE_TABS} tabsLabel="Finance pages" indicatorId="finance-nav-indicator">
      <Outlet />
    </ModulePageFrame>
  );
}
