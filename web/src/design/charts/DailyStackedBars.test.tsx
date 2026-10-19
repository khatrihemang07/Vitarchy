import { render, screen, within } from "@testing-library/react";
import { DailyStackedBars } from "./DailyStackedBars";
import type { StackSeries } from "./stackSeries";

const series: StackSeries<"active" | "away" | "unknown">[] = [
  { key: "active", label: "Active", look: "solid", color: "var(--module-accent)" },
  { key: "away", label: "Away", look: "hatched", color: "var(--away)" },
  { key: "unknown", label: "Away unknown", look: "faded", color: "var(--away-unknown)" },
];
const data = [
  { date: "2026-10-16", active: 3900, away: 600, unknown: 0 },
  { date: "2026-10-17", active: 0, away: 0, unknown: 1800 },
];
const format = (s: number) => `${String(s / 60)} min`;

describe("DailyStackedBars", () => {
  it("gives screen readers each Day's values as text", () => {
    render(<DailyStackedBars label="Timeline per Day" data={data} series={series} formatValue={format} />);
    const row = screen.getByRole("row", { name: /Fri 16 Oct/ });
    expect(within(row).getAllByRole("cell").map((c) => c.textContent)).toEqual(["65 min", "10 min", "0 min"]);
    expect(screen.getByRole("row", { name: /Sat 17 Oct/ })).toBeInTheDocument();
  });

  it("names the series in a legend, in the order given", () => {
    render(<DailyStackedBars label="Timeline per Day" data={data} series={series} formatValue={format} />);
    expect(screen.getAllByRole("listitem").map((li) => li.textContent)).toEqual(["Active", "Away", "Away unknown"]);
  });

  it("takes any series, stacked or side by side", () => {
    const money: StackSeries<"in" | "out">[] = [
      { key: "in", label: "Money in", look: "solid", color: "green" },
      { key: "out", label: "Money out", look: "hatched", color: "red" },
    ];
    render(<DailyStackedBars label="Money per Day" stacked={false} data={[{ date: "2026-10-16", in: 500, out: 200 }]} series={money} formatValue={(v) => `${String(v)} paise`} />);
    expect(screen.getAllByRole("listitem").map((li) => li.textContent)).toEqual(["Money in", "Money out"]);
    expect(within(screen.getByRole("row", { name: /Fri 16 Oct/ })).getAllByRole("cell").map((c) => c.textContent)).toEqual(["500 paise", "200 paise"]);
  });
});
