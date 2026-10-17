import { ChevronRight } from "lucide-react";
import type { Insight } from "@/api/time";
import { useCoreSettings } from "@/app/module-manifest";
import { INSIGHT_ICONS } from "./insightIcons";
import { insightTarget, type InsightTarget } from "./insightTarget";
import { describeInsight } from "./insightText";

interface InsightItemProps {
  insight: Insight;
  /** Days in each window a change compares. */
  windowDays: number;
  onOpen: (target: InsightTarget) => void;
}

/** One Insight as a sentence with its kind's icon; activating it opens the tab or Day it is about. */
export function InsightItem({ insight, windowDays, onOpen }: InsightItemProps) {
  const { clock } = useCoreSettings();
  const Icon = INSIGHT_ICONS[insight.kind];
  return (
    <li>
      <button
        type="button"
        onClick={() => { onOpen(insightTarget(insight)); }}
        className="flex w-full items-start gap-3 rounded-md px-1 py-2 text-left text-sm hover:bg-accent/50 focus-visible:outline-2 focus-visible:outline-ring"
      >
        <Icon aria-hidden className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
        <span className="min-w-0 flex-1">{describeInsight(insight, clock, windowDays)}</span>
        <ChevronRight aria-hidden className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
      </button>
    </li>
  );
}
