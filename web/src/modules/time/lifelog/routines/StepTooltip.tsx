import type { TooltipContentProps } from "recharts";
import { ChartTooltip, seriesColor } from "@/design";
import { formatDayLong } from "../../shared/dayLabel";
import { formatSpan } from "../../shared/duration";
import type { StepRow } from "./routineModel";

/** Hover card of the Step chart: each Step's exact length that Day, or that it did not happen. */
export function StepTooltip({ active, payload, labels }: TooltipContentProps & { labels: readonly string[] }) {
  const row = payload[0]?.payload as StepRow | undefined;
  if (!active || !row) return null;
  return (
    <ChartTooltip
      title={`${formatDayLong(row.date)}${row.complete ? "" : " (incomplete)"}`}
      rows={labels.map((label, index) => ({ label, color: seriesColor(index), value: row.seconds[index] == null ? "Missing" : formatSpan(row.seconds[index]) }))}
    />
  );
}
