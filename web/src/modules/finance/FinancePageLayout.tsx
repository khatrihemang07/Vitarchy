import { Outlet, useRouterState } from "@tanstack/react-router";
import { stagger } from "@/design";
import { FinanceNav } from "./FinanceNav";
import { financePageFor } from "./financePages";

/** The frame every Finance page shares, mounted once by the route above them: header, tabs and the page. */
export function FinancePageLayout() {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const { title, subtitle } = financePageFor(pathname);
  return (
    <div className="flex max-w-6xl flex-col gap-6">
      <header className="animate-fade-up">
        <h1 className="font-display text-4xl font-bold tracking-tight md:text-5xl">{title}</h1>
        <p className="text-muted-foreground">{subtitle}</p>
      </header>
      <div className="animate-fade-up" style={stagger(1)}>
        <FinanceNav />
      </div>
      <Outlet />
    </div>
  );
}
