import type { TooltipContentProps } from "recharts";
import { ChartTooltip } from "@/design";
import { formatDayLong } from "../shared/dayLabel";
import { formatSpan } from "../shared/duration";
import type { SplitRow, SplitSeries } from "./splitModel";

/** Hover card of the daily split: each project's exact time that Day, and what nobody logged. */
export function SplitTooltip({ active, payload, series }: TooltipContentProps & { series: readonly SplitSeries[] }) {
  const row = payload[0]?.payload as SplitRow | undefined;
  if (!active || !row) return null;
  const shown = series.filter((s) => (row.seconds[s.key] ?? 0) > 0);
  return (
    <ChartTooltip
      title={formatDayLong(row.date)}
      rows={(shown.length > 0 ? shown : series.slice(-1)).map((s) => ({ label: s.label, color: s.color, value: formatSpan(row.seconds[s.key] ?? 0) }))}
    />
  );
}
