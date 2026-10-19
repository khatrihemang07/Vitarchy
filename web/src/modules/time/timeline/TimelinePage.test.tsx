import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TooltipProvider } from "@/design";
import { jsonResponse, stubApi } from "../test-utils";
import { summary } from "./testSummary";
import { TimelinePage } from "./TimelinePage";
import { parseTimelineSearch, timelineParams } from "./timelineSearch";

afterEach(() => {
  vi.unstubAllGlobals();
});

const settings = { core: { week_start: "monday", clock: "h24" }, modules: {} };

function renderPage(path = "/time/timeline") {
  const root = createRootRoute();
  const route = createRoute({
    getParentRoute: () => root, path: "/time/timeline", component: TimelinePage,
    validateSearch: (raw: Record<string, unknown>) => timelineParams(parseTimelineSearch(raw)),
  });
  const sources = createRoute({ getParentRoute: () => root, path: "/time", component: () => null });
  const router = createRouter({ routeTree: root.addChildren([route, sources]), history: createMemoryHistory({ initialEntries: [path] }) });
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(<QueryClientProvider client={client}><TooltipProvider><RouterProvider router={router} /></TooltipProvider></QueryClientProvider>);
  return router;
}

const api = (timeline: () => Response | Promise<Response>) => ({
  "GET /api/settings": () => jsonResponse(200, settings),
  "GET /api/time/timeline": timeline,
});

const emptySummary = () => {
  const base = summary();
  return summary({ overview: { ...base.overview, active_secs: 0, away_secs: 0, unknown_secs: 0 }, apps: [], days: [], hours: [], usual: null });
};

describe("TimelinePage", () => {
  it("shows a loading notice, then the Timeline", async () => {
    stubApi(api(() => jsonResponse(200, summary())));
    renderPage();
    expect(await screen.findByText("Loading Timeline")).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "Overview" })).toBeInTheDocument();
  });

  it("explains a failure and tries again", async () => {
    stubApi(api(vi.fn().mockReturnValueOnce(jsonResponse(500, { detail: "boom" })).mockReturnValue(jsonResponse(200, summary()))));
    renderPage();
    await userEvent.click(await screen.findByRole("button", { name: "Try again" }));
    expect(await screen.findByRole("heading", { name: "Overview" })).toBeInTheDocument();
  });

  it("offers to Import Activity when the Period has no Timeline", async () => {
    stubApi(api(() => jsonResponse(200, emptySummary())));
    renderPage();
    expect(await screen.findByText("No Timeline for this Period yet. Import Activity on the Sources tab.")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Go to Sources" })).toHaveAttribute("href", "/time");
  });

  it("shows a Day as a band with its Segments, and sends the Period and time zone", async () => {
    const fetchMock = stubApi(api(() => jsonResponse(200, summary({}, "day"))));
    renderPage("/time/timeline?kind=day&date=2026-10-12");
    expect(await screen.findByRole("toolbar", { name: /Timeline of/ })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Segments" })).toBeInTheDocument();
    const sent = fetchMock.mock.calls.map(([r]) => new URL(r.url)).find((u) => u.pathname === "/api/time/timeline");
    expect(sent?.searchParams.get("kind")).toBe("day");
    expect(sent?.searchParams.get("from")).toBe("2026-10-12");
    expect(sent?.searchParams.get("tz")).toBeTruthy();
  });

  it("shows a Week as bars per Day without the band", async () => {
    stubApi(api(() => jsonResponse(200, summary())));
    renderPage("/time/timeline?date=2026-10-14");
    expect(await screen.findByRole("figure", { name: "Timeline per Day" })).toBeInTheDocument();
    expect(screen.queryByRole("toolbar")).not.toBeInTheDocument();
    expect(screen.getByRole("figure", { name: "Active time by hour of the day" })).toBeInTheDocument();
  });

  it("puts the highlighted app in the URL, and removes it again", async () => {
    stubApi(api(() => jsonResponse(200, summary())));
    const router = renderPage();
    await userEvent.click(await screen.findByRole("button", { name: "Safari" }));
    await waitFor(() => { expect(router.state.location.search).toMatchObject({ app: "Safari" }); });
    await userEvent.click(await screen.findByRole("button", { name: "Remove highlight" }));
    await waitFor(() => { expect(router.state.location.search).not.toHaveProperty("app"); });
  });
});
