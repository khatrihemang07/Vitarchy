import { render, screen } from "@testing-library/react";
import { DailyBars } from "./DailyBars";

describe("DailyBars", () => {
  it("gives screen readers each Day's time as text", () => {
    render(
      <DailyBars
        label="Time per Day"
        data={[{ date: "2026-10-16", seconds: 3900 }, { date: "2026-10-17", seconds: 0 }]}
        formatValue={(s) => `${String(s / 60)} min`}
      />,
    );
    expect(screen.getByRole("figure", { name: "Time per Day" })).toBeInTheDocument();
    expect(screen.getByRole("row", { name: "Fri 16 Oct 65 min" })).toBeInTheDocument();
    expect(screen.getByRole("row", { name: "Sat 17 Oct 0 min" })).toBeInTheDocument();
  });
});
