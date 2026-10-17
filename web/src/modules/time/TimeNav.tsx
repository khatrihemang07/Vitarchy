import { Link } from "@tanstack/react-router";
import { m } from "motion/react";
import { cn, springs } from "@/design";
import { TIME_TABS } from "./timePages";

/** Shared layout id so the active pill glides between the tabs. */
const INDICATOR_ID = "time-nav-indicator";

/** Switches between the Time pages: where data comes from, and the Reports built from it. */
export function TimeNav() {
  return (
    <nav aria-label="Time pages" className="flex w-fit gap-1">
      {TIME_TABS.map((link) => (
        <Link
          key={link.to}
          to={link.to}
          activeOptions={{ exact: true, includeSearch: false }}
          className={cn(
            "relative rounded-full px-3.5 py-2 text-sm font-medium text-muted-foreground transition-colors",
            "hover:text-foreground focus-ring data-[status=active]:text-foreground",
          )}
        >
          {({ isActive }) => (
            <>
              {isActive ? (
                <m.span
                  layoutId={INDICATOR_ID}
                  transition={springs.smooth}
                  className="absolute inset-0 rounded-full bg-accent ring-1 ring-module/40"
                />
              ) : null}
              <span className="relative">{link.label}</span>
            </>
          )}
        </Link>
      ))}
    </nav>
  );
}
