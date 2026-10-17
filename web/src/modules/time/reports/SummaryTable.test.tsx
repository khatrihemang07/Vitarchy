import { render } from "@testing-library/react";
import { TooltipProvider } from "@/design";
import { group } from "./testReport";
import { SummaryTable } from "./SummaryTable";

const context = { mode: "per_day", clock: "h24", periodDays: 7, isDay: false } as const;

describe("SummaryTable", () => {
  it("never clips its rows, so focus rings, tooltips and open details stay whole", () => {
    const { container } = render(
      <TooltipProvider>
        <SummaryTable groups={[group("Standup"), group("Review")]} context={context} />
      </TooltipProvider>,
    );
    // Only the containers: a progress bar inside a cell may clip its own fill.
    for (const el of container.querySelectorAll("section, section > div, ul, li")) expect(el.className).not.toContain("overflow-hidden");
  });

  it("rounds the bottom corners itself, for the last row to inherit", () => {
    const { container } = render(
      <TooltipProvider>
        <SummaryTable groups={[group("Standup")]} context={context} />
      </TooltipProvider>,
    );
    expect(container.querySelector("ul")).toHaveClass("rounded-b-xl");
    expect(container.querySelector("li")).toHaveClass("last:rounded-b-[inherit]");
  });
});
