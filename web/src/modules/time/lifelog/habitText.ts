/** A frequent description's habits written in words. Pure. */
import type { LifeLogDescriptionStat } from "@/api/time";
import { type Clock, formatMinuteOfDay } from "../shared/timeOfDay";
import { formatSpan } from "../shared/duration";

/** A usual value and the range most occurrences fall in. */
export interface HabitFigure {
  usual: string;
  range: string;
}

/** The sentence parts of one description's card. */
export interface HabitText {
  start: HabitFigure;
  end: HabitFigure;
  duration: HabitFigure;
  days: string;
}

const RANGE_SEPARATOR = " – ";

/** Start, end and length as "usual (range)" parts, and how many days it occurred on. */
export function describeHabit(stat: LifeLogDescriptionStat, clock: Clock): HabitText {
  const clockFigure = ({ median_minute, p25_minute, p75_minute }: LifeLogDescriptionStat["start"]): HabitFigure => ({
    usual: formatMinuteOfDay(median_minute, clock),
    range: `${formatMinuteOfDay(p25_minute, clock)}${RANGE_SEPARATOR}${formatMinuteOfDay(p75_minute, clock)}`,
  });
  const { median_seconds, p25_seconds, p75_seconds } = stat.duration;
  const days = `${String(stat.days)} ${stat.days === 1 ? "day" : "days"}`;
  return {
    start: clockFigure(stat.start),
    end: clockFigure(stat.end),
    duration: { usual: formatSpan(median_seconds), range: `${formatSpan(p25_seconds)}${RANGE_SEPARATOR}${formatSpan(p75_seconds)}` },
    days: stat.entries > stat.days ? `${days} (${String(stat.entries)} times)` : days,
  };
}
