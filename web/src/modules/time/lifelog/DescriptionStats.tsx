import type { ActivityPeriod } from "@/api/time";
import { useCoreSettings } from "@/app/module-manifest";
import { ChartPanel } from "../shared/ChartPanel";
import { HabitCard } from "./HabitCard";
import { useLifeLogPart } from "./useLifeLogPart";

/** Habits: the usual start, end and length of each description that keeps coming back. */
export function DescriptionStats({ period }: { period: ActivityPeriod }) {
  const { clock } = useCoreSettings();
  const query = useLifeLogPart(period, (overview) => overview.description_stats);
  return (
    <ChartPanel
      title="Your usual routine"
      description="For descriptions that keep coming back: the middle value, with the range most days fall in in brackets."
      query={query}
      isEmpty={(data) => data.length === 0}
      emptyMessage="No description was logged on enough days in this Period to show a pattern."
    >
      {(stats) => (
        <ul className="grid gap-3 sm:grid-cols-2">
          {stats.map((stat) => (
            <HabitCard key={stat.description} stat={stat} clock={clock} />
          ))}
        </ul>
      )}
    </ChartPanel>
  );
}
