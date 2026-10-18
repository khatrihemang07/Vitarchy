import { Link } from "@tanstack/react-router";
import { m } from "motion/react";
import { cn, springs } from "@/design";
import { FINANCE_TABS } from "./financePages";

/** Shared layout id so the active pill glides between the tabs. */
const INDICATOR_ID = "finance-nav-indicator";

/** Switches between the Finance pages: Import, Accounts and Transactions. */
export function FinanceNav() {
  return (
    <nav aria-label="Finance pages" className="flex w-fit max-w-full gap-1 overflow-x-auto">
      {FINANCE_TABS.map((link) => (
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
