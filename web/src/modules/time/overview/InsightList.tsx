import { useState } from "react";
import { type ActivityPeriod, useInsights } from "@/api/time";
import { Button } from "@/design";
import { ChartPanel } from "../shared/ChartPanel";
import { InsightItem } from "./InsightItem";
import type { InsightTarget } from "./insightTarget";

/** How many Insights lead the Overview before "Show all". */
export const HEADLINE_INSIGHTS = 5;

interface InsightListProps {
  period: ActivityPeriod;
  onOpen: (target: InsightTarget) => void;
}

/** The most notable Insights for the Period: the top few, and the rest on request. */
export function InsightList({ period, onOpen }: InsightListProps) {
  const query = useInsights(period);
  const [all, setAll] = useState(false);
  return (
    <ChartPanel
      title="Insights"
      description="Plain statistics from your own Time Entries and Activity, most notable first."
      query={query}
      isEmpty={(insights) => insights.length === 0}
      emptyMessage="Nothing stands out yet. Insights appear once there are enough Days of Time Entries or Activity."
    >
      {(insights) => (
        <>
          <ul className="flex flex-col">
            {(all ? insights : insights.slice(0, HEADLINE_INSIGHTS)).map((insight) => (
              <InsightItem key={`${insight.kind}-${insight.subject.kind}-${insight.subject.name}-${insight.measure}`} insight={insight} onOpen={onOpen} />
            ))}
          </ul>
          {insights.length > HEADLINE_INSIGHTS ? (
            <Button variant="ghost" size="sm" className="self-start" aria-expanded={all} onClick={() => { setAll(!all); }}>
              {all ? "Show fewer" : `Show all ${String(insights.length)}`}
            </Button>
          ) : null}
        </>
      )}
    </ChartPanel>
  );
}
