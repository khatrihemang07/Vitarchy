import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import { TooltipProvider } from "@/design";
import { summary } from "./testSummary";
import { TimelineOverview } from "./TimelineOverview";

/** Renders inside a router, since the Away tile links to the Sources page. */
function renderOverview(data = summary()) {
  const root = createRootRoute({ component: () => <TooltipProvider><TimelineOverview summary={data} /></TooltipProvider> });
  const sources = createRoute({ getParentRoute: () => root, path: "/time" });
  const router = createRouter({ routeTree: root.addChildren([sources]), history: createMemoryHistory({ initialEntries: ["/"] }) });
  return render(<RouterProvider router={router} />);
}

describe("TimelineOverview", () => {
  it("shows the five figures", async () => {
    renderOverview();
    for (const label of ["Active", "Away", "Away unknown", "Switches", "Longest focus"]) expect(await screen.findByText(label)).toBeInTheDocument();
    expect(screen.getByText("5h")).toBeInTheDocument();
  });

  it("links the Away tile to the Sources page, where the rules are shown", async () => {
    renderOverview();
    expect(await screen.findByRole("link", { name: "How Away is decided" })).toHaveAttribute("href", "/time");
    expect(screen.getAllByRole("link")).toHaveLength(1);
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
