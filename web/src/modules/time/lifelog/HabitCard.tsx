import type { LifeLogDescriptionStat } from "@/api/time";
import type { Clock } from "../shared/timeOfDay";
import { describeHabit } from "./habitText";

/** One description's habits: when it usually starts and ends, how long it lasts, and on how many days. */
export function HabitCard({ stat, clock }: { stat: LifeLogDescriptionStat; clock: Clock }) {
  const text = describeHabit(stat, clock);
  const lines = [
    { label: "Usually starts", figure: text.start },
    { label: "Usually ends", figure: text.end },
    { label: "Lasts", figure: text.duration },
  ];
  return (
    <li className="flex min-w-0 flex-col gap-2 rounded-lg border bg-background/40 p-3">
      <div className="flex items-baseline justify-between gap-2">
        <h3 className="truncate font-medium" title={stat.description}>{stat.description}</h3>
        <span className="shrink-0 text-xs text-muted-foreground">{text.days}</span>
      </div>
      <dl className="flex flex-col gap-1 text-sm">
        {lines.map(({ label, figure }) => (
          <div key={label} className="flex flex-wrap items-baseline gap-x-2">
            <dt className="text-muted-foreground">{label}</dt>
            <dd className="tabular-nums">
              <span className="font-medium">{figure.usual}</span> <span className="text-xs text-muted-foreground">({figure.range})</span>
            </dd>
          </div>
        ))}
      </dl>
    </li>
  );
}
