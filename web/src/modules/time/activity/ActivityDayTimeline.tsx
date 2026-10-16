import { useState } from "react";
import { type ActivityPeriod, useActivityTimeline } from "@/api/time";
import { useCoreSettings } from "@/app/module-manifest";
import { DayTimeline } from "../shared/DayTimeline";
import { ChartPanel } from "../shared/ChartPanel";
import { pickDay } from "../shared/dates";
import { DayPicker } from "../shared/DayPicker";
import { formatDayLong } from "../shared/dayLabel";
import { formatSpan } from "../shared/duration";
import { useActivityPart } from "./useActivityPart";

/** One Day of Activity as a lane of Contexts. Starts on the last Day with Activity in the Period. */
export function ActivityDayTimeline({ period }: { period: ActivityPeriod }) {
  const { clock } = useCoreSettings();
  const bounds = useActivityPart(period, (overview) => overview.bounds);
  const [chosen, setChosen] = useState<string>();
  const activeDays = bounds.data?.map((day) => day.date) ?? [];
  const day = pickDay(chosen, period.from, period.to, activeDays);
  const query = useActivityTimeline(day, period.tz);
  const title = "A day in detail";
  if (!day) {
    // No Day has Activity (or the Days are still loading): the Period's own bounds answer for the panel.
    return (
      <ChartPanel title={title} query={bounds} isEmpty={() => true} emptyMessage="No Activity in this Period.">
        {() => null}
      </ChartPanel>
    );
  }
  return (
    <ChartPanel
      title={title}
      description={`${formatDayLong(day)}: what the computer was showing, hour by hour.`}
      actions={<DayPicker day={day} from={period.from} to={period.to} onChange={setChosen} />}
      query={query}
      isEmpty={(data) => data.spans.length === 0}
      emptyMessage="No Activity on this day."
    >
      {(data) => (
        <DayTimeline
          dayStart={data.day_start}
          dayEnd={data.day_end}
          tz={period.tz}
          clock={clock}
          subject="Activity"
          formatSeconds={formatSpan}
          segments={data.spans.map((span) => ({
            start: span.start,
            end: span.end,
            label: span.context,
            ...(span.context === span.app ? {} : { detail: span.app }),
          }))}
        />
      )}
    </ChartPanel>
  );
}
