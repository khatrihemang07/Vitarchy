import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ModuleTabs } from "./ModuleTabs";

const TABS = [
  { to: "/finance", label: "Import" },
  { to: "/finance/accounts", label: "Accounts" },
  { to: "/finance/transactions", label: "Transactions" },
];

function renderAt(path: string) {
  const root = createRootRoute({ component: () => <ModuleTabs tabs={TABS} label="Finance pages" indicatorId="test-indicator" /> });
  const routes = TABS.map((tab) => createRoute({ getParentRoute: () => root, path: tab.to }));
  const router = createRouter({ routeTree: root.addChildren(routes), history: createMemoryHistory({ initialEntries: [path] }) });
  return render(<RouterProvider router={router} />);
}

describe("ModuleTabs", () => {
  it("marks Accounts as the current page on its route", async () => {
    renderAt("/finance/accounts");
    expect(await screen.findByRole("link", { name: "Accounts" })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: "Import" })).not.toHaveAttribute("aria-current");
  });

  it("keeps Transactions current while its filters are in the address", async () => {
    renderAt("/finance/transactions?account=3");
    expect(await screen.findByRole("link", { name: "Transactions" })).toHaveAttribute("aria-current", "page");
  });

  it("marks only the first tab as current on its own path", async () => {
    renderAt("/finance");
    expect(await screen.findByRole("link", { name: "Import" })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: "Accounts" })).not.toHaveAttribute("aria-current");
  });

  it("names the navigation", async () => {
    renderAt("/finance");
    expect(await screen.findByRole("navigation", { name: "Finance pages" })).toBeInTheDocument();
  });
});

describe("ModuleTabs behaviour", () => {
  it("is not an overflow container, so the active pill's ring and the focus ring are never clipped", async () => {
    renderAt("/finance");
    const nav = await screen.findByRole("navigation", { name: "Finance pages" });
    expect(nav.className).not.toMatch(/overflow/);
  });

  it("starts no request when a tab is hovered; only opening the tab does", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    try {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const loader = vi.fn();
      const root = createRootRoute({ component: () => <ModuleTabs tabs={TABS} label="Finance pages" indicatorId="hover-indicator" /> });
      const routes = TABS.map((tab) => createRoute({ getParentRoute: () => root, path: tab.to, ...(tab.to === "/finance/accounts" ? { loader } : {}) }));
      const router = createRouter({ routeTree: root.addChildren(routes), history: createMemoryHistory({ initialEntries: ["/finance"] }) });
      render(<RouterProvider router={router} />);

      await user.hover(await screen.findByRole("link", { name: "Accounts" }));
      await vi.advanceTimersByTimeAsync(1000);
      expect(loader).not.toHaveBeenCalled();

      await user.click(screen.getByRole("link", { name: "Accounts" }));
      await waitFor(() => { expect(loader).toHaveBeenCalled(); });
    } finally {
      vi.useRealTimers();
    }
  });
});
