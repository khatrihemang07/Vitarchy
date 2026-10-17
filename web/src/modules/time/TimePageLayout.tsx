import { Outlet, useRouterState } from "@tanstack/react-router";
import { stagger } from "@/design";
import { TimeNav } from "./TimeNav";
import { timePageFor } from "./timePages";

/**
 * The frame every Time page shares, mounted once by the route above them: one width, then
 * header, tabs and the page, so nothing jumps and the tab pill can glide between pages.
 */
export function TimePageLayout() {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const { title, subtitle } = timePageFor(pathname);
  return (
    <div className="flex max-w-6xl flex-col gap-6">
      <header className="animate-fade-up">
        <h1 className="font-display text-4xl font-bold tracking-tight md:text-5xl">{title}</h1>
        <p className="text-muted-foreground">{subtitle}</p>
      </header>
      <div className="animate-fade-up" style={stagger(1)}>
        <TimeNav />
      </div>
      <Outlet />
    </div>
  );
}
