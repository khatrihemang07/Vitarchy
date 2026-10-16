/** Where an Insight leads: the tab that shows its subject, and the Day it names, if any. Pure. */
import type { Insight, InsightSubjectKind } from "@/api/time";
import type { TimeTabId } from "../tabs";

/** The tab and (optionally) the single Day to open. */
export interface InsightTarget {
  tab: TimeTabId;
  day?: string;
}

/** Screen time lives in Activity; Routines and descriptions live in the Life log. */
const TAB_OF_SUBJECT: Record<InsightSubjectKind, TimeTabId> = {
  routine_step: "life-log",
  routine_total: "life-log",
  description: "life-log",
  app: "activity",
  context: "activity",
};

/** Where to go to see more of `insight`. */
export function insightTarget(insight: Insight): InsightTarget {
  const day = insight.days[0];
  return { tab: TAB_OF_SUBJECT[insight.subject.kind], ...(day !== undefined && { day }) };
}
