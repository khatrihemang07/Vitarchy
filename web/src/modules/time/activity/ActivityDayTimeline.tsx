import { useState } from "react";
import { type ActivityPeriod, useActivityBounds, useActivityTimeline } from "@/api/time";
import { useCoreSettings } from "@/app/module-manifest";
import { DayTimeline } from "../shared/DayTimeline";
import { ActivityPanel } from "./ActivityPanel";
import { pickDay } from "./dates";
import { DayPicker } from "./DayPicker";
import { formatDayLong } from "./dayLabel";
import { formatSpan } from "./duration";

/** One Day of Activity as a lane of Contexts. Starts on the last Day with Activity in the Period. */
export function ActivityDayTimeline({ period }: { period: ActivityPeriod }) {
  const { clock } = useCoreSettings();
  const bounds = useActivityBounds(period);
  const [chosen, setChosen] = useState<string>();
  const activeDays = bounds.data?.days.map((day) => day.date) ?? [];
  const day = pickDay(chosen, period.from, period.to, activeDays);
  const query = useActivityTimeline(day, period.tz);
  const title = "A day in detail";
  if (!day) {
    // No Day has Activity (or the Days are still loading): the Period's own bounds answer for the panel.
    return (
      <ActivityPanel title={title} query={bounds} isEmpty={() => true} emptyMessage="No Activity in this Period.">
        {() => null}
      </ActivityPanel>
    );
  }
  return (
    <ActivityPanel
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
    </ActivityPanel>
  );
}
