import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRootRoute, createRouter, Outlet, RouterProvider } from "@tanstack/react-router";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { timeModule } from "./index";
import { disconnected, jsonResponse, stubApi } from "./test-utils";

async function renderAt(path: string) {
  stubApi({
    "GET /api/time/status": () => jsonResponse(200, disconnected),
    "GET /api/settings": () => jsonResponse(404, { detail: "none" }),
  });
  const root = createRootRoute({ component: Outlet });
  const router = createRouter({
    routeTree: root.addChildren(timeModule.routes(root)),
    history: createMemoryHistory({ initialEntries: [path] }),
  });
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  await screen.findByRole("heading", { name: "Time" });
  return router;
}

describe("TimePage", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("opens on the Overview with a neutral note, and the Sources cards stay out of the way", async () => {
    await renderAt("/time");
    expect(screen.getByRole("tab", { name: "Overview", selected: true })).toBeInTheDocument();
    expect(screen.getByText(/once there is data for this Period/)).toBeInTheDocument();
    expect(screen.queryByText("Toggl connection")).not.toBeInTheDocument();
  });

  it("shows the connection, Sync and Import cards under Sources, without a period picker", async () => {
    await renderAt("/time?tab=sources");
    expect(await screen.findByText("Toggl connection")).toBeInTheDocument();
    expect(screen.queryByRole("radiogroup", { name: "Period" })).not.toBeInTheDocument();
  });

  it("keeps the tab and period in the URL", async () => {
    const router = await renderAt("/time?tab=activity&period=month&from=2026-03-01");
    expect(screen.getByRole("radio", { name: "Month", checked: true })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("tab", { name: "Life log" }));
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ tab: "life-log", period: "month", from: "2026-03-01" });
    });
  });

  it("opens a bad link on the defaults instead of failing", async () => {
    await renderAt("/time?tab=nope&period=decade&from=garbage");
    expect(screen.getByRole("tab", { name: "Overview", selected: true })).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: "Week", checked: true })).toBeInTheDocument();
  });

  it("rolls today over at local midnight while the page stays open", async () => {
    vi.useFakeTimers({ toFake: ["Date", "setInterval", "clearInterval"], now: new Date(2026, 9, 14, 23, 59, 50) });
    try {
      await renderAt("/time?period=custom&from=2026-10-01&to=2026-10-02");
      expect(screen.getByLabelText("From")).toHaveAttribute("max", "2026-10-14");
      await act(async () => { await vi.advanceTimersByTimeAsync(31_000); });
      expect(screen.getByLabelText("From")).toHaveAttribute("max", "2026-10-15");
    } finally {
      vi.useRealTimers();
    }
  });
});
