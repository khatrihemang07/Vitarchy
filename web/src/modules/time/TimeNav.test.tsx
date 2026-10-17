import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import { TimeNav } from "./TimeNav";

function renderAt(path: string) {
  const root = createRootRoute({ component: TimeNav });
  const routes = ["/time", "/time/reports"].map((p) => createRoute({ getParentRoute: () => root, path: p }));
  const router = createRouter({ routeTree: root.addChildren(routes), history: createMemoryHistory({ initialEntries: [path] }) });
  return render(<RouterProvider router={router} />);
}

describe("TimeNav", () => {
  it("marks Reports as the current page on the Reports route", async () => {
    renderAt("/time/reports");
    expect(await screen.findByRole("link", { name: "Reports" })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: "Sources" })).not.toHaveAttribute("aria-current");
  });

  it("keeps Reports current while its filters are in the address", async () => {
    renderAt("/time/reports?kind=day&date=2026-10-13");
    expect(await screen.findByRole("link", { name: "Reports" })).toHaveAttribute("aria-current", "page");
  });

  it("marks only Sources as current on /time, not the Reports route under it", async () => {
    renderAt("/time");
    expect(await screen.findByRole("link", { name: "Sources" })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: "Reports" })).not.toHaveAttribute("aria-current");
  });
});
