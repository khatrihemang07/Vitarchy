import type { DailyFigure } from "@/api/finance";
import { DailyStackedBars, decimalTicks, type StackSeries } from "@/design";
import { formatInr, formatInrCompact } from "../money";

/** Money out is hatched, so the two read apart without colour. */
const SERIES: StackSeries<"money_in" | "money_out">[] = [
  { key: "money_in", label: "Money in", look: "solid", color: "var(--success)" },
  { key: "money_out", label: "Money out", look: "hatched", color: "var(--destructive)" },
];

/** Money in and Money out for each day, side by side (stacking them would add up two opposite things). */
export function MoneyBars({ daily }: { daily: readonly DailyFigure[] }) {
  const data = daily.map((d) => ({ date: d.date, money_in: d.money_in_paise, money_out: d.money_out_paise }));
  return <DailyStackedBars label="Money in and Money out per day" data={data} series={SERIES} stacked={false} formatValue={formatInr} formatAxis={formatInrCompact} ticksFor={decimalTicks} />;
}
