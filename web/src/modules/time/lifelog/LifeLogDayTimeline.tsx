import { useState } from "react";
import { type ActivityPeriod, useLifeLogOverview, useLifeLogTimeline } from "@/api/time";
import { OTHER_COLOR } from "@/design";
import { useCoreSettings } from "@/app/module-manifest";
import { ChartPanel } from "../shared/ChartPanel";
import { pickDay } from "../shared/dates";
import { DayPicker } from "../shared/DayPicker";
import { formatDayLong } from "../shared/dayLabel";
import { formatSpan } from "../shared/duration";
import { DayTimeline } from "../shared/DayTimeline";
import { useProjectColors } from "./useProjectColors";

/** One Day of Time Entries as a lane: coloured by project, labelled by description. Starts on the last Day with entries. */
export function LifeLogDayTimeline({ period }: { period: ActivityPeriod }) {
  const { clock } = useCoreSettings();
  const split = useLifeLogOverview(period, (overview) => overview.split);
  const colors = useProjectColors(period);
  const [chosen, setChosen] = useState<string>();
  const activeDays = split.data?.filter((day) => day.by_project.length > 0).map((day) => day.date) ?? [];
  const day = pickDay(chosen, period.from, period.to, activeDays);
  // The lane waits for the project colours, so it never shows colours the other panels do not.
  const query = useLifeLogTimeline(colors ? day : undefined, period.tz);
  const title = "A day in detail";
  if (!day) {
    // No Day has entries (or the Days are still loading): the Period's own split answers for the panel.
    return (
      <ChartPanel title={title} query={split} isEmpty={() => true} emptyMessage="No Time Entries in this Period.">
        {() => null}
      </ChartPanel>
    );
  }
  return (
    <ChartPanel
      title={title}
      description={`${formatDayLong(day)}: what you logged, hour by hour. Gaps are unlogged.`}
      actions={<DayPicker day={day} from={period.from} to={period.to} onChange={setChosen} />}
      query={split.isError ? { ...query, isPending: false, isError: true, error: split.error } : query}
      isEmpty={(data) => data.spans.length === 0}
      emptyMessage="No Time Entries on this day."
    >
      {(data) => (
        <DayTimeline
          dayStart={data.day_start}
          dayEnd={data.day_end}
          tz={period.tz}
          clock={clock}
          subject="Time Entries"
          formatSeconds={formatSpan}
          segments={data.spans.map((span) => ({
            start: span.start,
            end: span.end,
            label: span.description,
            group: span.project,
            detail: span.project,
            color: colors?.get(span.project) ?? OTHER_COLOR,
          }))}
        />
      )}
    </ChartPanel>
  );
}
