import { render, screen } from "@testing-library/react";
import { TooltipProvider } from "@/design";
import { summary } from "./testSummary";
import { TimelineOverview } from "./TimelineOverview";

const renderOverview = (data = summary()) => render(<TooltipProvider><TimelineOverview summary={data} /></TooltipProvider>);

describe("TimelineOverview", () => {
  it("shows the five figures", () => {
    renderOverview();
    for (const label of ["Active", "Away", "Away unknown", "Switches", "Longest focus"]) expect(screen.getByText(label)).toBeInTheDocument();
    expect(screen.getByText("5h")).toBeInTheDocument();
  });

  it("badges each figure's change from the Usual and says what it is compared with", () => {
    renderOverview();
    expect(screen.getAllByText(/^[↑↓]/).length).toBeGreaterThanOrEqual(4);
    expect(screen.getByText(/^Compared with the last 4 weeks/)).toBeInTheDocument();
  });

  it("writes the Longest focus comparison on its tile", () => {
    renderOverview();
    expect(screen.getByText("vs your best Day in the last 4 weeks")).toBeInTheDocument();
  });

  it("shows no badge without a Usual, and says why", () => {
    renderOverview(summary({ usual: null }));
    expect(screen.queryByText(/^[↑↓]/)).not.toBeInTheDocument();
    expect(screen.getByText(/Nothing to compare with yet/)).toBeInTheDocument();
  });
});
