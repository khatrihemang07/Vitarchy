import type { DayTotals } from "@/api/time";
import { DailyStackedBars, type StackSeries } from "@/design";
import { formatSpan } from "../format";

/** Away is hatched and unknown Away faded, so the three read apart without colour. */
const SERIES: StackSeries<"active" | "away" | "unknown">[] = [
  { key: "active", label: "Active", look: "solid", color: "var(--module-accent)" },
  { key: "away", label: "Away", look: "hatched", color: "var(--away)" },
  { key: "unknown", label: "Away unknown", look: "faded", color: "var(--away-unknown)" },
];

/** Active, Away and Away unknown time for each Day of a longer Period. */
export function TimelineDays({ days }: { days: readonly DayTotals[] }) {
  const data = days.map((d) => ({ date: d.date, active: d.active_secs, away: d.away_secs, unknown: d.unknown_secs }));
  return <DailyStackedBars label="Timeline per Day" data={data} series={SERIES} formatValue={formatSpan} />;
}
