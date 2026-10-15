import { Link } from "@tanstack/react-router";
import { motion } from "motion/react";
import type { ComponentType } from "react";
import { accentScope, cn, Pressable, springs } from "@/design";

interface NavItemProps {
  to: string;
  label: string;
  icon: ComponentType<{ className?: string }>;
  active: boolean;
  /** Any CSS colour; scopes `text-module` / the indicator to this entry. */
  accent?: string;
}

/** Shared layout id so the active pill glides between entries. */
const INDICATOR_ID = "nav-indicator";

export function NavItem({ to, label, icon: Icon, active, accent }: NavItemProps) {
  return (
    <div style={accent ? accentScope(accent) : {}} className="shrink-0">
      <Pressable>
        <Link
          to={to}
          aria-current={active ? "page" : undefined}
          className={cn(
            "relative flex items-center gap-2.5 rounded-full px-3.5 py-2 text-sm font-medium",
            "text-muted-foreground transition-colors hover:text-foreground",
            "focus-ring",
            active && "text-foreground",
          )}
        >
          {active ? (
            <motion.span
              layoutId={INDICATOR_ID}
              transition={springs.smooth}
              className="absolute inset-0 rounded-full bg-accent ring-1 ring-module/40"
            />
          ) : null}
          <Icon className="relative size-4 text-module" />
          <span className="relative">{label}</span>
        </Link>
      </Pressable>
    </div>
  );
}
