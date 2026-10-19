import { Link } from "@tanstack/react-router";
import { m } from "motion/react";
import { cn } from "../lib/utils";
import { springs } from "../motion/springs";

/** One tab: where it goes and what it says. */
export interface ModuleTab {
  to: string;
  label: string;
}

interface ModuleTabsProps {
  tabs: readonly ModuleTab[];
  /** Accessible name of the navigation ("Finance pages"). */
  label: string;
  /** Shared layout id, so the active pill glides between the tabs; unique per Module. */
  indicatorId: string;
}

/**
 * The tab pill that switches between a Module's pages; the active tab moves at once, before the
 * page's data arrives. It renders router links, which is routing, not data access, so it lives in the design system.
 */
export function ModuleTabs({ tabs, label, indicatorId }: ModuleTabsProps) {
  return (
    <nav aria-label={label} className="flex w-fit gap-1">
      {tabs.map((tab) => (
        <Link
          key={tab.to}
          to={tab.to}
          activeOptions={{ exact: true, includeSearch: false }}
          className={cn(
            "relative rounded-full px-3.5 py-2 text-sm font-medium text-muted-foreground transition-colors",
            "hover:text-foreground focus-ring data-[status=active]:text-foreground",
          )}
        >
          {({ isActive }) => (
            <>
              {isActive ? (
                <m.span layoutId={indicatorId} transition={springs.smooth} className="absolute inset-0 rounded-full bg-accent ring-1 ring-module/40" />
              ) : null}
              <span className="relative">{tab.label}</span>
            </>
          )}
        </Link>
      ))}
    </nav>
  );
}
