import { render, screen, within } from "@testing-library/react";
import { DailyStackedBars } from "./DailyStackedBars";

const data = [
  { date: "2026-10-16", active: 3900, away: 600, unknown: 0 },
  { date: "2026-10-17", active: 0, away: 0, unknown: 1800 },
];
const format = (s: number) => `${String(s / 60)} min`;

describe("DailyStackedBars", () => {
  it("gives screen readers each Day's Active, Away and unknown time as text", () => {
    render(<DailyStackedBars label="Timeline per Day" data={data} formatValue={format} />);
    const row = screen.getByRole("row", { name: /Fri 16 Oct/ });
    expect(within(row).getAllByRole("cell").map((c) => c.textContent)).toEqual(["65 min", "10 min", "0 min"]);
    expect(screen.getByRole("row", { name: /Sat 17 Oct/ })).toBeInTheDocument();
  });

  it("names the three series in a legend", () => {
    render(<DailyStackedBars label="Timeline per Day" data={data} formatValue={format} />);
    const legend = screen.getAllByRole("listitem").map((li) => li.textContent);
    expect(legend).toEqual(["Active", "Away", "Away unknown"]);
  });
});
