import { hourTicks } from "./bandLayout";
import type { Clock } from "../reports/entryTimes";
import { formatHour } from "./timelineFormat";

/** Hours between the labels on a wide screen; a narrow one shows every second label. */
const STEP_HOURS = 3;
const NARROW_STEP_HOURS = 6;

/** The hour marks under (and lines through) a 24 h band, in the clock format from Settings. */
export function BandTicks({ day, clock }: { day: string; clock: Clock }) {
  const ticks = hourTicks(day, STEP_HOURS);
  return (
    <div aria-hidden="true" className="relative mt-1 h-4 text-[0.7rem] text-muted-foreground tabular-nums">
      {ticks.map(({ pct, date }) => {
        const wideOnly = date.getHours() % NARROW_STEP_HOURS !== 0 && pct !== 100;
        const align = pct === 0 ? "" : pct === 100 ? "-translate-x-full" : "-translate-x-1/2";
        return (
          <span key={pct} className={`absolute top-0 whitespace-nowrap ${align} ${wideOnly ? "hidden sm:block" : ""}`} style={{ left: `${String(pct)}%` }}>
            {formatHour(date, clock)}
          </span>
        );
      })}
    </div>
  );
}
