import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import { FinanceNav } from "./FinanceNav";

function renderAt(path: string) {
  const root = createRootRoute({ component: FinanceNav });
  const routes = ["/finance", "/finance/accounts", "/finance/transactions"].map((p) =>
    createRoute({ getParentRoute: () => root, path: p }),
  );
  const router = createRouter({ routeTree: root.addChildren(routes), history: createMemoryHistory({ initialEntries: [path] }) });
  return render(<RouterProvider router={router} />);
}

describe("FinanceNav", () => {
  it("marks Accounts as the current page on its route", async () => {
    renderAt("/finance/accounts");
    expect(await screen.findByRole("link", { name: "Accounts" })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: "Import" })).not.toHaveAttribute("aria-current");
  });

  it("keeps Transactions current while its filters are in the address", async () => {
    renderAt("/finance/transactions?account=3");
    expect(await screen.findByRole("link", { name: "Transactions" })).toHaveAttribute("aria-current", "page");
  });

  it("marks only Import as current on /finance", async () => {
    renderAt("/finance");
    expect(await screen.findByRole("link", { name: "Import" })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: "Accounts" })).not.toHaveAttribute("aria-current");
  });
});
