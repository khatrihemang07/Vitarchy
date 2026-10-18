import type { DayTotals } from "@/api/time";
import { DailyStackedBars } from "@/design";
import { formatSpan } from "../format";

/** Active, Away and Away unknown time for each Day of a longer Period. */
export function TimelineDays({ days }: { days: readonly DayTotals[] }) {
  const data = days.map((d) => ({ date: d.date, active: d.active_secs, away: d.away_secs, unknown: d.unknown_secs }));
  return <DailyStackedBars label="Timeline per Day" data={data} formatValue={formatSpan} />;
}
