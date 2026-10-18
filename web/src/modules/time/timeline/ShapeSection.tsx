import type { TimelineSummary } from "@/api/time";
import type { Clock } from "../reports/entryTimes";
import { FocusCard } from "./FocusCard";
import { HourStrip } from "./HourStrip";
import { TimelineSection } from "./TimelineSection";

interface ShapeSectionProps {
  summary: TimelineSummary;
  clock: Clock;
  isDay: boolean;
}

/** The shape of the Period: Focus and Switching beside Active time by hour. */
export function ShapeSection({ summary, clock, isDay }: ShapeSectionProps) {
  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <TimelineSection title="Focus and Switching"><FocusCard overview={summary.overview} clock={clock} isDay={isDay} /></TimelineSection>
      <TimelineSection title="Active by hour"><HourStrip hours={summary.hours} clock={clock} /></TimelineSection>
    </div>
  );
}
