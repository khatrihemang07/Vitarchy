import { ChevronRight } from "lucide-react";
import type { AppUsage } from "@/api/time";
import { cn, Expand } from "@/design";
import { formatSpan } from "../format";
import { Breakdown } from "./Breakdown";
import { formatShare } from "./timelineFormat";

interface AppRowProps {
  usage: AppUsage;
  color: string;
  open: boolean;
  highlighted: boolean;
  onToggleOpen: () => void;
  onHighlight: () => void;
}

/** One app: a disclosure for its window titles and sites, a button to highlight it, and its share of Active time. */
export function AppRow({ usage, color, open, highlighted, onToggleOpen, onHighlight }: AppRowProps) {
  const panelId = `app-${usage.app.replace(/\W+/g, "-")}`;
  return (
    <li className={cn(highlighted && "bg-accent/50")}>
      <div className="flex items-center gap-2 px-4 py-2">
        <button type="button" aria-expanded={open} aria-controls={panelId} aria-label={`Details of ${usage.app}`} onClick={onToggleOpen} className="grid size-6 shrink-0 place-items-center rounded-sm text-muted-foreground hover:text-foreground focus-ring">
          <ChevronRight aria-hidden="true" className={cn("size-4 transition-transform", open && "rotate-90")} />
        </button>
        <span aria-hidden="true" className="size-2.5 shrink-0 rounded-full" style={{ background: color }} />
        <button type="button" aria-pressed={highlighted} onClick={onHighlight} className="min-w-0 flex-1 truncate rounded-sm text-left text-sm font-medium hover:underline focus-ring">
          {usage.app || "(no app)"}
        </button>
        <div aria-hidden="true" className="hidden h-2 w-32 shrink-0 overflow-hidden rounded-full bg-muted sm:block md:w-48">
          <div className="h-full rounded-full" style={{ width: `${String(Math.max(2, usage.share * 100))}%`, background: color }} />
        </div>
        <span className="w-20 shrink-0 whitespace-nowrap text-right text-sm font-medium tabular-nums">{formatSpan(usage.active_secs)}</span>
        <span className="w-10 shrink-0 text-right text-xs text-muted-foreground tabular-nums">{formatShare(usage.share)}</span>
      </div>
      <Expand open={open} id={panelId}>
        <div className="grid gap-4 px-4 pb-3 pl-14 sm:grid-cols-2">
          <Breakdown heading="Window titles" rows={usage.titles.map((t) => ({ name: t.title, secs: t.active_secs }))} />
          <Breakdown heading="Sites" rows={usage.sites.map((s) => ({ name: s.domain, secs: s.active_secs }))} />
          {usage.titles.length + usage.sites.length === 0 ? <p className="text-sm text-muted-foreground">No titles or sites recorded.</p> : null}
        </div>
      </Expand>
    </li>
  );
}
