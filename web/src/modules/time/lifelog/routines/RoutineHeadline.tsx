import type { RoutineStats, RoutineStep } from "@/api/time";
import { useCoreSettings } from "@/app/module-manifest";
import { formatSpan } from "../../shared/duration";
import { formatMinuteOfDay } from "../../shared/timeOfDay";

/** The Routine at a glance: the usual door-to-door time, then each Step's usual start and length. */
export function RoutineHeadline({ steps, stats }: { steps: readonly RoutineStep[]; stats: RoutineStats }) {
  const { clock } = useCoreSettings();
  const total = stats.door_to_door;
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm">
        {total ? (
          <>
            <span className="text-2xl font-semibold tabular-nums">{formatSpan(total.median_seconds)}</span>{" "}
            <span className="text-muted-foreground">
              door to door, usually {formatSpan(total.p25_seconds)} to {formatSpan(total.p75_seconds)}, over {String(stats.complete_days)} complete {stats.complete_days === 1 ? "day" : "days"}
            </span>
          </>
        ) : (
          <span className="text-muted-foreground">No complete day yet, so there is no door-to-door time.</span>
        )}
      </p>
      <ol className="grid gap-2 sm:grid-cols-2">
        {steps.map((step, index) => {
          const stat = stats.steps[index];
          const wait = stats.gaps[index];
          return (
            <li key={`${String(index)}-${step.label}`} className="min-w-0 rounded-lg border bg-background/40 p-3 text-sm">
              <h4 className="truncate font-medium" title={step.label}>{step.label}</h4>
              {stat?.duration && stat.start ? (
                <p className="tabular-nums">
                  <span className="font-medium">{formatSpan(stat.duration.median_seconds)}</span>{" "}
                  <span className="text-xs text-muted-foreground">
                    from about {formatMinuteOfDay(stat.start.median_minute, clock)}, on {String(stat.days)} {stat.days === 1 ? "day" : "days"}
                  </span>
                </p>
              ) : (
                <p className="text-xs text-muted-foreground">Never happened in this Period.</p>
              )}
              {wait ? <p className="text-xs text-muted-foreground">Then a wait of {formatSpan(wait.median_seconds)}.</p> : null}
            </li>
          );
        })}
      </ol>
    </div>
  );
}
