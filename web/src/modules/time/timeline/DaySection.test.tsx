import { render, screen, within } from "@testing-library/react";
import { TooltipProvider } from "@/design";
import { DaySection } from "./DaySection";
import { appColorMap } from "./appColors";
import { summary } from "./testSummary";

const data = summary({}, "day");

function renderSection() {
  return render(
    <TooltipProvider>
      <DaySection summary={data} colors={appColorMap(data.apps)} clock="h24" now={new Date(2026, 9, 13, 12)} onApp={vi.fn()} />
    </TooltipProvider>,
  );
}

describe("DaySection", () => {
  it("labels the Time Entries lane and the Activity band", () => {
    renderSection();
    expect(screen.getByText("Activity")).toBeInTheDocument();
    expect(screen.getByRole("group", { name: /^Time Entries on this Day/ })).toBeInTheDocument();
    expect(screen.getAllByText("Time Entries")).toHaveLength(2);
  });

  it("explains the lane's colour in the legend", () => {
    renderSection();
    expect(within(screen.getByRole("list", { name: "Colours" })).getByText("Time Entries")).toBeInTheDocument();
  });
});
