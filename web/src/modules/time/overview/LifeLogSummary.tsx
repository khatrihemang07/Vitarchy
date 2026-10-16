import type { ActivityPeriod, LifeLogOverview } from "@/api/time";
import { useLifeLogOverview } from "@/api/time";
import { OTHER_COLOR } from "@/design";
import { useProjectColors } from "../lifelog/useProjectColors";
import { ChartPanel } from "../shared/ChartPanel";
import { formatShare, formatSpan } from "../shared/duration";
import { CompactBars } from "./CompactBars";
import { unloggedShare } from "./overviewModel";
import { RoutineSummary } from "./RoutineSummary";

/** Projects listed on the Overview. */
const TOP_PROJECTS = 5;

// Defined once so the query memoizes the selection.
const selectSummary = (overview: LifeLogOverview) => ({
  projects: overview.totals.by_project.slice(0, TOP_PROJECTS).map((p) => ({ name: p.name, seconds: p.seconds })),
  logged: overview.totals.logged_seconds,
  ...unloggedShare(overview.split),
});

/** The Life log for the Period: hours per project, the Unlogged time, and each Routine's headline. */
export function LifeLogSummary({ period }: { period: ActivityPeriod }) {
  const query = useLifeLogOverview(period, selectSummary);
  const colors = useProjectColors(period);
  return (
    <ChartPanel
      title="Life log"
      description="Logged time by project. Overlapping entries each count in full."
      query={query}
      isEmpty={(summary) => summary.projects.length === 0}
      emptyMessage="No Time Entries in this Period."
    >
      {(summary) => (
        <>
          <CompactBars label="Hours per project" rows={summary.projects} total={summary.logged} colorOf={(name) => colors?.get(name) ?? OTHER_COLOR} />
          <p className="text-sm">
            <span className="font-medium tabular-nums">{formatSpan(summary.unlogged)}</span>{" "}
            <span className="text-muted-foreground">Unlogged, {formatShare(summary.unlogged, summary.elapsed)} of the time that has passed</span>
          </p>
          <section aria-label="Routines" className="flex flex-col gap-2 border-t pt-3">
            <h3 className="text-sm font-medium text-muted-foreground">Routines</h3>
            <RoutineSummary period={period} />
          </section>
        </>
      )}
    </ChartPanel>
  );
}
