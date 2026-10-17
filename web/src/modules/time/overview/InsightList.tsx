import { useState } from "react";
import { type ActivityPeriod, type Insight, useInsights } from "@/api/time";
import { Button } from "@/design";
import { ChartPanel } from "../shared/ChartPanel";
import { InsightItem } from "./InsightItem";
import type { InsightTarget } from "./insightTarget";

/** How many Insights lead the Overview before "Show all". */
export const HEADLINE_INSIGHTS = 5;

/** A key that tells Insights apart even when two Routines share a Step label; the index is the last resort. */
const insightKey = (insight: Insight, index: number) =>
  [insight.kind, insight.subject.kind, insight.subject.routine?.id ?? "", insight.subject.name, insight.measure, index].join("-");

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
      isEmpty={(result) => result.insights.length === 0}
      emptyMessage="Nothing stands out yet. Insights appear once there are enough Days of Time Entries or Activity."
    >
      {({ insights, change_window_days: windowDays }) => (
        <>
          <ul className="flex flex-col">
            {(all ? insights : insights.slice(0, HEADLINE_INSIGHTS)).map((insight, index) => (
              <InsightItem key={insightKey(insight, index)} insight={insight} windowDays={windowDays} onOpen={onOpen} />
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
