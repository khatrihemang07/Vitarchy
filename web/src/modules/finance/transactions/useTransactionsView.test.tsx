import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { render, waitFor } from "@testing-library/react";
import { account, jsonResponse, overview, stubApi } from "../test-utils";
import { parseTransactionsSearch, transactionsParams } from "./transactionsSearch";
import { useTransactionsView } from "./useTransactionsView";

afterEach(() => vi.unstubAllGlobals());

function Probe({ onView }: { onView: (view: ReturnType<typeof useTransactionsView>) => void }) {
  onView(useTransactionsView());
  return null;
}

function mount(path: string, settings: () => Response | Promise<Response>) {
  const fetchMock = stubApi({
    "GET /api/settings": settings,
    "GET /api/finance/accounts": () => jsonResponse(200, [account()]),
    "GET /api/finance/transactions": () => jsonResponse(200, { total: 0, items: [] }),
    "GET /api/finance/overview": () => jsonResponse(200, overview()),
  });
  let latest: ReturnType<typeof useTransactionsView> | undefined;
  const root = createRootRoute();
  const route = createRoute({
    getParentRoute: () => root,
    path: "/finance/transactions",
    validateSearch: (raw: Record<string, unknown>) => transactionsParams(parseTransactionsSearch(raw)),
    component: () => <Probe onView={(v) => { latest = v; }} />,
  });
  const router = createRouter({ routeTree: root.addChildren([route]), history: createMemoryHistory({ initialEntries: [path] }) });
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return { fetchMock, view: () => latest };
}

const asked = (fetchMock: ReturnType<typeof stubApi>, suffix: string) =>
  fetchMock.mock.calls.map(([r]) => new URL(r.url)).filter((u) => u.pathname.endsWith(suffix));

describe("useTransactionsView", () => {
  it("waits for the week start before asking for a week, then asks once", async () => {
    let release: (r: Response) => void = () => undefined;
    const { fetchMock } = mount("/finance/transactions?kind=week&date=2026-10-14", () => new Promise<Response>((resolve) => { release = resolve; }));
    await waitFor(() => { expect(asked(fetchMock, "/accounts")).toHaveLength(1); });
    expect(asked(fetchMock, "/transactions")).toHaveLength(0);
    expect(asked(fetchMock, "/overview")).toHaveLength(0);

    release(jsonResponse(200, { core: { week_start: "sunday" } }));
    await waitFor(() => { expect(asked(fetchMock, "/overview")).toHaveLength(1); });
    expect(asked(fetchMock, "/overview")[0]?.searchParams.get("from")).toBe("2026-10-11");
    expect(asked(fetchMock, "/transactions")).toHaveLength(1);
  });

  it("does not wait for Settings when the Period does not depend on the week start", async () => {
    const { fetchMock, view } = mount("/finance/transactions?kind=month&date=2026-10-14", () => new Promise<Response>(() => undefined));
    await waitFor(() => { expect(asked(fetchMock, "/transactions")).toHaveLength(1); });
    expect(view()?.range).toEqual({ from: "2026-10-01", to: "2026-10-31" });
  });

  it("says whether the shown Period holds today, except for a custom stretch", async () => {
    const { view } = mount("/finance/transactions?kind=custom&from=2020-01-01&to=2099-01-01", () => jsonResponse(200, { core: {} }));
    await waitFor(() => { expect(view()).toBeDefined(); });
    expect(view()?.isCurrent).toBe(false);
  });
});
