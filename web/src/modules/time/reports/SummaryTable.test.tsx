import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TooltipProvider } from "@/design";
import { rowContextFor } from "./rowContext";
import { group, report } from "./testReport";
import { SummaryTable } from "./SummaryTable";

const contextFor = (kind: "week" | "day", compare: "previous" | "usual" = "previous") =>
  rowContextFor({ period: report([], kind).period, mode: "per_day", compare, clock: "h24" });
const context = contextFor("week");

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

  it("opens no row by default", () => {
    render(
      <TooltipProvider>
        <SummaryTable groups={[group("Standup"), group("Review")]} context={context} />
      </TooltipProvider>,
    );
    for (const button of screen.getAllByRole("button")) expect(button).toHaveAttribute("aria-expanded", "false");
  });

  it("heads the columns Avg, Min and Max per Day, and explains one on hover", async () => {
    render(
      <TooltipProvider>
        <SummaryTable groups={[group("Standup")]} context={context} />
      </TooltipProvider>,
    );
    for (const heading of ["Avg per Day", "Min per Day", "Max per Day"]) expect(screen.getByText(heading, { selector: "span[tabindex]" })).toBeInTheDocument();
    await userEvent.hover(screen.getByText("Min per Day", { selector: "span[tabindex]" }));
    expect((await screen.findAllByText("The smallest daily total among the Days this group happened")).length).toBeGreaterThan(0);
  });

  it("heads a Day's columns Usual and Usual range against the last 4 weeks", () => {
    render(
      <TooltipProvider>
        <SummaryTable groups={[group("Standup")]} context={contextFor("day", "usual")} />
      </TooltipProvider>,
    );
    expect(screen.getByText("Usual range", { selector: "span[tabindex]" })).toBeInTheDocument();
    expect(screen.queryByText("Avg per Day", { selector: "span[tabindex]" })).not.toBeInTheDocument();
  });
});
