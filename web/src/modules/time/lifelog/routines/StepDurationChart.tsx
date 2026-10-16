import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, type TooltipContentProps, XAxis, YAxis } from "recharts";
import type { RoutineOccurrence, RoutineStep } from "@/api/time";
import { axisProps, ChartLegend, gridProps, SEGMENT_GAP, seriesColor, tooltipProps } from "@/design";
import { ChartTable } from "../../shared/ChartTable";
import { formatDayShort } from "../../shared/dayLabel";
import { formatSpan } from "../../shared/duration";
import { stepKey, stepRows } from "./routineModel";
import { StepTooltip } from "./StepTooltip";

const MAX_BAR = 24;

/** How long each Step took on each Day, as stacked bars, with the same numbers as a table. */
export function StepDurationChart({ steps, occurrences }: { steps: readonly RoutineStep[]; occurrences: readonly RoutineOccurrence[] }) {
  const rows = stepRows(occurrences);
  const labels = steps.map((step) => step.label);
  return (
    <>
      <ChartLegend items={labels.map((label, index) => ({ label, color: seriesColor(index) }))} />
      <div role="img" aria-label="Stacked bar chart of how long each Step took on each Day. The same numbers follow as a table." className="h-56 w-full min-w-0">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={rows} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
            <CartesianGrid {...gridProps} />
            <XAxis dataKey="date" {...axisProps} tickFormatter={formatDayShort} interval="preserveStartEnd" minTickGap={24} />
            <YAxis {...axisProps} width={44} tickFormatter={(h: number) => `${String(h)} h`} />
            <Tooltip {...tooltipProps} content={(props: TooltipContentProps) => <StepTooltip {...props} labels={labels} />} />
            {labels.map((_, index) => (
              <Bar key={stepKey(index)} dataKey={stepKey(index)} stackId="day" fill={seriesColor(index)} stroke="var(--card)" strokeWidth={SEGMENT_GAP / 2} maxBarSize={MAX_BAR} isAnimationActive={false} />
            ))}
          </BarChart>
        </ResponsiveContainer>
      </div>
      <ChartTable
        caption="Length of each Step for each Day"
        columns={["Day", ...labels]}
        rows={rows.map((row) => [formatDayShort(row.date), ...row.seconds.map((s) => (s === null ? "Missing" : formatSpan(s)))])}
      />
    </>
  );
}
