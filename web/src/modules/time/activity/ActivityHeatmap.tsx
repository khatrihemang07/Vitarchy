import { type ActivityPeriod, useActivityHeatmap } from "@/api/time";
import { useCoreSettings } from "@/app/module-manifest";
import { ActivityPanel } from "./ActivityPanel";
import { ChartTable } from "../shared/ChartTable";
import { formatSpan } from "./duration";
import { cellFill, HeatmapGrid } from "./HeatmapGrid";
import { maxCell, orderRows, peakCell } from "./heatmapModel";
import { formatHourOfDay } from "../shared/timeOfDay";

const SCALE = [0, 0.25, 0.5, 0.75, 1];

/** When the computer is in use: seconds of Activity by weekday and hour, summed over the Period. */
export function ActivityHeatmap({ period }: { period: ActivityPeriod }) {
  const query = useActivityHeatmap(period);
  const { week_start: weekStart, clock } = useCoreSettings();
  return (
    <ActivityPanel
      title="When you are on the computer"
      description="Time of day by weekday, added up over the Period. Darker is more."
      query={query}
      isEmpty={(data) => data.seconds.every((day) => day.every((cell) => cell === 0))}
      emptyMessage="No Activity in this Period."
    >
      {(data) => {
        const rows = orderRows(data.seconds, weekStart);
        const peak = peakCell(rows);
        return (
          <>
            {peak ? (
              <p className="text-sm text-muted-foreground">
                Busiest: {peak.row.name}s around {formatHourOfDay(peak.hour, clock)} ({formatSpan(peak.seconds)} in all).
              </p>
            ) : null}
            <HeatmapGrid rows={rows} clock={clock} />
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              Less
              {SCALE.map((strength) => (
                <span key={strength} aria-hidden className="size-3 rounded-sm" style={{ background: cellFill(strength).fill, opacity: cellFill(strength).fillOpacity }} />
              ))}
              More
              <span className="ml-auto">max {formatSpan(maxCell(rows))} in an hour slot</span>
            </div>
            <ChartTable
              caption="Minutes of Activity by hour of the day and weekday"
              columns={["Hour", ...rows.map((row) => row.short)]}
              rows={Array.from({ length: 24 }, (_, hour) => [
                formatHourOfDay(hour, clock),
                ...rows.map((row) => String(Math.round((row.hours[hour] ?? 0) / 60))),
              ])}
            />
          </>
        );
      }}
    </ActivityPanel>
  );
}
