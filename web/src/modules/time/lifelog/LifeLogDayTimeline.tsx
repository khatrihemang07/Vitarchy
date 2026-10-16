import { useState } from "react";
import { type ActivityPeriod, useLifeLogTimeline } from "@/api/time";
import { useCoreSettings } from "@/app/module-manifest";
import { ChartPanel } from "../shared/ChartPanel";
import { pickDay } from "../shared/dates";
import { DayPicker } from "../shared/DayPicker";
import { formatDayLong } from "../shared/dayLabel";
import { formatSpan } from "../shared/duration";
import { DayTimeline } from "../shared/DayTimeline";
import { useLifeLogPart, useProjectColors } from "./useLifeLogPart";

/** One Day of Time Entries as a lane: coloured by project, labelled by description. Starts on the last Day with entries. */
export function LifeLogDayTimeline({ period }: { period: ActivityPeriod }) {
  const { clock } = useCoreSettings();
  const split = useLifeLogPart(period, (overview) => overview.split);
  const colors = useProjectColors(period);
  const [chosen, setChosen] = useState<string>();
  const activeDays = split.data?.filter((day) => day.by_project.length > 0).map((day) => day.date) ?? [];
  const day = pickDay(chosen, period.from, period.to, activeDays);
  const query = useLifeLogTimeline(day, period.tz);
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
      query={query}
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
          segments={data.spans.map((span) => {
            const color = colors?.get(span.project);
            return { start: span.start, end: span.end, label: span.description, group: span.project, detail: span.project, ...(color ? { color } : {}) };
          })}
        />
      )}
    </ChartPanel>
  );
}
