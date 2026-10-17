import { Link } from "@tanstack/react-router";
import { cn } from "@/design";

const LINKS = [
  { to: "/time", label: "Sources" },
  { to: "/time/reports", label: "Reports" },
] as const;

/** Switches between the Time pages: where data comes from, and the Reports built from it. */
export function TimeNav() {
  return (
    <nav aria-label="Time pages" className="flex gap-1 border-b border-border">
      {LINKS.map((link) => (
        <Link
          key={link.to}
          to={link.to}
          activeOptions={{ exact: true }}
          className={cn(
            "-mb-px border-b-2 border-transparent px-3 py-2 text-sm font-medium text-muted-foreground transition-colors",
            "hover:text-foreground focus-ring data-[status=active]:border-module data-[status=active]:text-foreground",
          )}
        >
          {link.label}
        </Link>
      ))}
    </nav>
  );
}
