import { HourHeat } from "@/design";
import { formatSpan } from "../format";
import type { Clock } from "../reports/entryTimes";
import { formatHour } from "./timelineFormat";

/** Active time by hour of the day over the Period, so the shape of the day shows at a glance. */
export function HourStrip({ hours, clock }: { hours: readonly number[]; clock: Clock }) {
  return (
    <HourHeat
      label="Active time by hour of the day"
      values={hours}
      hourLabel={(hour) => formatHour(new Date(2026, 0, 1, hour), clock)}
      formatValue={formatSpan}
    />
  );
}
