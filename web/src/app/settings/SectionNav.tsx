import { Link } from "@tanstack/react-router";
import { cn } from "@/design";
import type { SettingsSection } from "./schema";

interface SectionNavProps {
  sections: readonly SettingsSection[];
  activeId: string;
}

/** Section list: a column on desktop, a scrolling tab strip on phones. */
export function SectionNav({ sections, activeId }: SectionNavProps) {
  return (
    <nav
      aria-label="Settings sections"
      className="-mx-4 flex gap-1 overflow-x-auto px-4 pb-1 md:mx-0 md:w-52 md:shrink-0 md:flex-col md:overflow-visible md:px-0"
    >
      {sections.map(({ id, label, icon: Icon }) => (
        <Link
          key={id}
          to="/settings/$section"
          params={{ section: id }}
          aria-current={id === activeId ? "page" : undefined}
          className={cn(
            "flex shrink-0 items-center gap-2.5 rounded-full px-3.5 py-2 text-sm font-medium",
            "text-muted-foreground transition-colors hover:text-foreground focus-ring",
            id === activeId && "bg-accent text-foreground ring-1 ring-module/40",
          )}
        >
          <Icon className="size-4 text-module" />
          {label}
        </Link>
      ))}
    </nav>
  );
}
