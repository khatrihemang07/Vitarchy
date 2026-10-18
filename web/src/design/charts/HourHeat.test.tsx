import { render, screen } from "@testing-library/react";
import { TooltipProvider } from "../ui/tooltip";
import { HourHeat } from "./HourHeat";

const values = Array.from({ length: 24 }, (_, h) => (h === 9 ? 3600 : 0));

function renderHeat() {
  return render(
    <TooltipProvider>
      <HourHeat label="Active time by hour" values={values} hourLabel={(h) => `${String(h)}h`} formatValue={(s) => `${String(s / 60)} min`} />
    </TooltipProvider>,
  );
}

describe("HourHeat", () => {
  it("gives screen readers all 24 hours as text", () => {
    renderHeat();
    expect(screen.getAllByRole("row")).toHaveLength(24);
    expect(screen.getByRole("row", { name: "9h 60 min" })).toBeInTheDocument();
    expect(screen.getByRole("row", { name: "3h 0 min" })).toBeInTheDocument();
  });

  it("is named for its purpose", () => {
    renderHeat();
    expect(screen.getByRole("figure", { name: "Active time by hour" })).toBeInTheDocument();
  });
});
