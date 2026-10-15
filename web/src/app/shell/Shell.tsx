import { Outlet, useRouterState } from "@tanstack/react-router";
import { accentScope } from "@/design";
import { resolveAccent } from "../accent";
import type { NavEntry } from "../nav";
import { CommandBar } from "./CommandBar";
import { Sidebar } from "./Sidebar";

/**
 * App frame. Exposes the active Module's accent as `--module-accent`, so any
 * descendant (`text-module`, `bg-module/20`, primary buttons, focus rings)
 * picks it up with no wiring.
 */
export function Shell({ nav }: { nav: readonly NavEntry[] }) {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const { moduleId, accent } = resolveAccent(pathname, nav);

  return (
    <div
      data-module={moduleId ?? "home"}
      style={accentScope(accent)}
      className="bg-glow bg-grain min-h-dvh md:grid md:grid-cols-shell"
    >
      <Sidebar nav={nav} activeModuleId={moduleId} homeActive={pathname === "/"} />
      <div className="flex min-w-0 flex-col">
        <header className="flex items-center p-4 md:px-8 md:pt-6">
          <CommandBar nav={nav} />
        </header>
        <main className="flex-1 p-4 pb-12 md:px-8">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
