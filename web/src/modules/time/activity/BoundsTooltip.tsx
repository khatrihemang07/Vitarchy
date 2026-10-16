import type { TooltipContentProps } from "recharts";
import { ChartTooltip } from "@/design";
import type { BoundsPoint } from "./chartModels";
import { formatDayShort } from "./dayLabel";
import { type Clock, formatMinuteOfDay } from "../shared/timeOfDay";

/** Hover card of the first and last Activity chart; `first` and `last` name the line colours. */
export function BoundsTooltip({ active, payload, clock, first, last }: TooltipContentProps & { clock: Clock; first: string; last: string }) {
  const point = payload[0]?.payload as BoundsPoint | undefined;
  if (!active || !point) return null;
  const time = (minute: number | null) => (minute === null ? "No Activity" : formatMinuteOfDay(minute, clock));
  return (
    <ChartTooltip
      title={formatDayShort(point.date)}
      rows={[
        { label: "First activity", color: first, value: time(point.first) },
        { label: "Last activity", color: last, value: time(point.last) },
      ]}
    />
  );
}
