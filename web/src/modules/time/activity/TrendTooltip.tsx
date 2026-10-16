import type { TooltipContentProps } from "recharts";
import { ChartTooltip } from "@/design";
import { secondsKey, type TrendRow, type TrendSeriesInfo } from "./chartModels";
import { formatDayShort } from "../shared/dayLabel";
import { formatSpan } from "../shared/duration";

/** Hover card of the weekly trend: each series' exact time that week. */
export function TrendTooltip({ active, payload, series }: TooltipContentProps & { series: readonly TrendSeriesInfo[] }) {
  const row = payload[0]?.payload as TrendRow | undefined;
  if (!active || !row) return null;
  const title = `Week of ${formatDayShort(row.start)}${row.days < 7 ? ` (${String(row.days)} of 7 days)` : ""}`;
  return (
    <ChartTooltip
      title={title}
      rows={series.map((s) => ({ label: s.label, color: s.color, value: formatSpan(Number(row[secondsKey(s.key)] ?? 0)) }))}
    />
  );
}
