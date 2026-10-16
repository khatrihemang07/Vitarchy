import { useState } from "react";
import type { ActivityPeriod } from "@/api/time";
import { SegmentedControl } from "@/design";
import { ChartPanel } from "../shared/ChartPanel";
import { DescriptionRow } from "./DescriptionRow";
import { ProjectRow } from "./ProjectRow";
import { useLifeLogPart, useProjectColors } from "./useLifeLogPart";

const VIEWS = [
  { value: "project", label: "By project" },
  { value: "description", label: "By description" },
] as const;

/** Logged time by project (open one for its descriptions) or by description across projects. */
export function TotalsTable({ period }: { period: ActivityPeriod }) {
  const query = useLifeLogPart(period, (overview) => overview.totals);
  const colors = useProjectColors(period);
  const [view, setView] = useState<string>(VIEWS[0].value);
  return (
    <ChartPanel
      title="Where your logged time goes"
      description="Each entry counts its own time, so overlapping entries add up to more than the clock. Shares are of that total."
      query={query}
      isEmpty={(data) => data.by_project.length === 0}
      emptyMessage="No Time Entries in this Period."
      actions={<SegmentedControl label="Group totals" value={view} onValueChange={setView} options={VIEWS} />}
    >
      {(totals) => (
        <div className="overflow-x-auto">
          <table className="w-full min-w-72 table-fixed text-left">
            <caption className="sr-only">Logged time {view === "project" ? "by project" : "by description"}</caption>
            <colgroup>
              <col />
              <col className="w-28" />
              <col className="hidden w-14 sm:table-column" />
              <col className="hidden w-12 sm:table-column" />
              <col className="w-12" />
            </colgroup>
            <thead>
              <tr className="text-xs text-muted-foreground">
                <th scope="col" className="py-1 font-medium">{view === "project" ? "Project" : "Description"}</th>
                <th scope="col" className="py-1 pl-3 text-right font-medium">Time</th>
                <th scope="col" className="hidden py-1 pl-3 text-right font-medium sm:table-cell">Entries</th>
                <th scope="col" className="hidden py-1 pl-3 text-right font-medium sm:table-cell">Days</th>
                <th scope="col" className="py-1 pl-3 text-right font-medium">Share</th>
              </tr>
            </thead>
            <tbody>
              {view === "project"
                ? totals.by_project.map((project) => (
                    <ProjectRow key={project.name} project={project} color={colors?.get(project.name) ?? "var(--chart-other)"} loggedSeconds={totals.logged_seconds} />
                  ))
                : totals.by_description.map((total) => <DescriptionRow key={total.description} total={total} loggedSeconds={totals.logged_seconds} />)}
            </tbody>
          </table>
        </div>
      )}
    </ChartPanel>
  );
}
