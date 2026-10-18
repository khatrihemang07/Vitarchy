import { render, screen } from "@testing-library/react";
import { TooltipProvider } from "@/design";
import { summary } from "./testSummary";
import { TimelineOverview } from "./TimelineOverview";

function renderOverview(data = summary()) {
  return render(<TooltipProvider><TimelineOverview summary={data} /></TooltipProvider>);
}

describe("TimelineOverview", () => {
  it("shows the five figures", async () => {
    renderOverview();
    for (const label of ["Active", "Away", "Away unknown", "Switches", "Longest focus"]) expect(await screen.findByText(label)).toBeInTheDocument();
    expect(screen.getByText("5h")).toBeInTheDocument();
  });

  it("shows no link on any tile", async () => {
    renderOverview();
    await screen.findByText("Away");
    expect(screen.queryByRole("link")).not.toBeInTheDocument();
  });

  it("badges each figure's change from the Usual and says what it is compared with", async () => {
    renderOverview();
    expect((await screen.findAllByText(/^[↑↓]/)).length).toBeGreaterThanOrEqual(4);
    expect(screen.getByText(/^Compared with the last 4 weeks/)).toBeInTheDocument();
  });

  it("writes the Longest focus comparison on its tile", async () => {
    renderOverview();
    expect(await screen.findByText("vs your best Day in the last 4 weeks")).toBeInTheDocument();
  });

  it("shows no badge without a Usual, and says why", async () => {
    renderOverview(summary({ usual: null }));
    expect(await screen.findByText(/Nothing to compare with yet/)).toBeInTheDocument();
    expect(screen.queryByText(/^[↑↓]/)).not.toBeInTheDocument();
    expect(screen.getByText(/Nothing to compare with yet/)).toBeInTheDocument();
  });
});
