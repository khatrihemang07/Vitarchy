import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TooltipProvider } from "@/design";
import { modules } from "./modules";
import { createAppRouter } from "./router";

afterEach(() => vi.unstubAllGlobals());

function renderAt(path: string) {
  // Every request hangs, so a page that waits for its data would leave the old tab active.
  vi.stubGlobal("fetch", vi.fn(() => new Promise<Response>(() => undefined)));
  const router = createAppRouter(modules, { includeDesignShowcase: false });
  router.update({ history: createMemoryHistory({ initialEntries: [path] }) });
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <TooltipProvider>
        <RouterProvider router={router} />
      </TooltipProvider>
    </QueryClientProvider>,
  );
  return router;
}

describe("tab navigation", () => {
  it.each([
    ["Time", "/time", "Time pages", "Reports", "/time/reports"],
    ["Finance", "/finance", "Finance pages", "Transactions", "/finance/transactions"],
  ])("moves the active %s tab and the URL at once, while the new page's queries are still pending", async (_module, start, nav, tab, target) => {
    const router = renderAt(start);
    const tabs = await screen.findByRole("navigation", { name: nav });
    await userEvent.click(await screen.findByRole("link", { name: tab }));

    expect(screen.getByRole("link", { name: tab })).toHaveAttribute("aria-current", "page");
    expect(router.state.location.pathname).toBe(target);
    expect(tabs).toBeInTheDocument();
    expect(await screen.findByRole("heading", { level: 1, name: tab })).toBeInTheDocument();
  });
});
