import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ReportsPage } from "./ReportsPage";
import { parseReportSearch, toSearchParams } from "./reportSearch";
import { disconnected, jsonResponse, stubApi, synced } from "../test-utils";
import { group, report } from "./testReport";

afterEach(() => {
  vi.unstubAllGlobals();
  localStorage.clear();
});

const facets = { projects: [{ id: 1, name: "Office", active: true, last_used: "2026-10-01T00:00:00Z" }], tags: [], clients: [] };
const settings = { core: { week_start: "monday", clock: "h24" }, modules: {} };

function renderPage(path = "/time/reports?project=%5B1%5D") {
  const root = createRootRoute();
  const route = createRoute({
    getParentRoute: () => root, path: "/time/reports", component: ReportsPage,
    validateSearch: (raw: Record<string, unknown>) => toSearchParams(parseReportSearch(raw)),
  });
  const sources = createRoute({ getParentRoute: () => root, path: "/time", component: () => null });
  const router = createRouter({ routeTree: root.addChildren([route, sources]), history: createMemoryHistory({ initialEntries: [path] }) });
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(<QueryClientProvider client={client}><RouterProvider router={router} /></QueryClientProvider>);
}

const api = (reportHandler: () => Response | Promise<Response>, status = synced) => ({
  "GET /api/settings": () => jsonResponse(200, settings),
  "GET /api/time/status": () => jsonResponse(200, status),
  "GET /api/time/facets": () => jsonResponse(200, facets),
  "GET /api/time/report": reportHandler,
});

describe("ReportsPage", () => {
  it("shows a loading notice, then the Report with its first row open", async () => {
    stubApi(api(() => jsonResponse(200, report([group("Standup"), group("Review")]))));
    renderPage();
    expect(await screen.findByText("Loading report…")).toBeInTheDocument();
    expect(await screen.findByRole("button", { name: /Standup/ })).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("button", { name: /Review/ })).toHaveAttribute("aria-expanded", "false");
  });

  it("sends the filters, the Period and the time zone, repeating list values", async () => {
    const fetchMock = stubApi(api(() => jsonResponse(200, report([group("Standup")]))));
    renderPage("/time/reports?project=%5B1%2C2%5D&date=2026-10-16");
    await screen.findByRole("button", { name: /Standup/ });
    const sent = fetchMock.mock.calls.map(([r]) => new URL(r.url)).find((u) => u.pathname === "/api/time/report");
    expect(sent?.searchParams.getAll("project")).toEqual(["1", "2"]);
    expect(sent?.searchParams.get("from")).toBe("2026-10-12");
    expect(sent?.searchParams.get("to")).toBe("2026-10-18");
    expect(sent?.searchParams.get("tz")).toBeTruthy();
  });

  it("explains a failure", async () => {
    stubApi(api(() => jsonResponse(400, { status: 400, detail: "bad period" })));
    renderPage();
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });

  it("offers to clear filters when nothing matches", async () => {
    stubApi(api(() => jsonResponse(200, report([]))));
    renderPage();
    expect(await screen.findByText("No Time Entries match")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Clear filters" }).length).toBeGreaterThan(0);
  });

  it("sends the person to the Time page when Toggl was never connected", async () => {
    stubApi(api(() => jsonResponse(200, report([])), disconnected));
    renderPage();
    expect(await screen.findByText("Toggl is not connected yet.")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Connect Toggl/ })).toHaveAttribute("href", "/time");
  });

  it("shows the default week start with a Try again that reloads Settings", async () => {
    vi.spyOn(console, "warn").mockImplementation(() => undefined);
    let up = false;
    stubApi({ ...api(() => jsonResponse(200, report([group("Standup")]))), "GET /api/settings": () => (up ? jsonResponse(200, settings) : jsonResponse(500, { title: "boom" })) });
    renderPage();
    expect(await screen.findByText(/Using the default week start/)).toBeInTheDocument();
    expect(await screen.findByRole("button", { name: /Standup/ })).toBeInTheDocument();
    up = true;
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    await waitFor(() => { expect(screen.queryByText(/Using the default week start/)).not.toBeInTheDocument(); });
  });

  it("shows no Settings notice when Settings load", async () => {
    stubApi(api(() => jsonResponse(200, report([group("Standup")]))));
    renderPage();
    await screen.findByRole("button", { name: /Standup/ });
    expect(screen.queryByText(/Using the default week start/)).not.toBeInTheDocument();
  });

  it("opens unfiltered on a first visit, with no project chosen for the person", async () => {
    const fetchMock = stubApi(api(() => jsonResponse(200, report([group("Standup")]))));
    renderPage("/time/reports");
    await screen.findByRole("button", { name: /Standup/ });
    const sent = fetchMock.mock.calls.map(([r]) => new URL(r.url)).find((u) => u.pathname === "/api/time/report");
    expect(sent?.searchParams.getAll("project")).toEqual([]);
  });
});
