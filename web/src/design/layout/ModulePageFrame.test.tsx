import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import { ModulePageFrame } from "./ModulePageFrame";

function renderFrame() {
  const root = createRootRoute({
    component: () => (
      <ModulePageFrame title="Accounts" subtitle="Your Accounts." tabs={[{ to: "/a", label: "Alpha" }]} tabsLabel="Test pages" indicatorId="frame-test">
        <p>the page</p>
      </ModulePageFrame>
    ),
  });
  const router = createRouter({ routeTree: root.addChildren([createRoute({ getParentRoute: () => root, path: "/a" })]), history: createMemoryHistory({ initialEntries: ["/a"] }) });
  return render(<RouterProvider router={router} />);
}

describe("ModulePageFrame", () => {
  it("puts the header, the tabs and the page in one frame", async () => {
    renderFrame();
    expect(await screen.findByRole("heading", { level: 1, name: "Accounts" })).toBeInTheDocument();
    expect(screen.getByRole("navigation", { name: "Test pages" })).toBeInTheDocument();
    expect(screen.getByText("the page")).toBeInTheDocument();
  });
});
