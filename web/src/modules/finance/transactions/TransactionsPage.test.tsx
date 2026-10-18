import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { account, jsonResponse, pendingForever, SETTLED_NOTICES, stubApi } from "../test-utils";
import { parseTransactionsSearch, transactionsParams } from "../transactionsSearch";
import { TransactionsPage } from "./TransactionsPage";

afterEach(() => vi.unstubAllGlobals());

function renderAt(path: string) {
  const root = createRootRoute();
  const route = createRoute({
    getParentRoute: () => root,
    path: "/finance/transactions",
    validateSearch: (raw: Record<string, unknown>) => transactionsParams(parseTransactionsSearch(raw)),
    component: TransactionsPage,
  });
  const router = createRouter({ routeTree: root.addChildren([route]), history: createMemoryHistory({ initialEntries: [path] }) });
  return render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

const item = (over: Record<string, unknown> = {}) => ({
  id: 1, account_id: 1, date: "2026-04-02", description: "Groceries", amount_paise: 12_345_678, direction: "debit",
  balance_paise: 50_000_000, no_longer_found: false, ...over,
});
const page = (items: unknown[], total = items.length) => () => jsonResponse(200, { total, items });

describe("TransactionsPage", () => {
  it("shows only a loading notice, never an empty or failure one, while the queries are pending", async () => {
    stubApi({ "GET /api/finance/accounts": pendingForever, "GET /api/finance/transactions": pendingForever });
    const { container } = renderAt("/finance/transactions");
    await screen.findByText("Loading Transactions…");
    expect(container.querySelectorAll(SETTLED_NOTICES)).toHaveLength(0);
  });

  it("shows amounts in INR with Indian grouping and the Account's name", async () => {
    stubApi({ "GET /api/finance/accounts": () => jsonResponse(200, [account()]), "GET /api/finance/transactions": page([item()]) });
    renderAt("/finance/transactions");
    expect(await screen.findByText("−₹1,23,456.78")).toHaveAttribute("data-direction", "debit");
    expect(screen.getByText("₹5,00,000.00")).toBeInTheDocument();
    expect(await screen.findByText("HDFC Savings", { selector: "td" })).toBeInTheDocument();
  });

  it("shows the linked Payment record inline and flags rows no longer found", async () => {
    stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, [account()]),
      "GET /api/finance/transactions": page([
        item({ payment_record: { id: 5, occurred_at: "2026-04-02T18:30:00", description: "Paid to Fresh Mart", utr: "412345678901" } }),
        item({ id: 2, description: "Old row", no_longer_found: true }),
      ]),
    });
    renderAt("/finance/transactions");
    expect(await screen.findByText("Paid to Fresh Mart")).toBeInTheDocument();
    expect(screen.getByText("UTR 412345678901")).toBeInTheDocument();
    expect(screen.getByText("2 Apr 2026, 18:30")).toBeInTheDocument();
    expect(screen.getByText("No longer found")).toBeInTheDocument();
  });

  it("asks the server only for the filters in the address", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, [account()]),
      "GET /api/finance/transactions": page([item()], 120),
    });
    renderAt("/finance/transactions?account=1&from=2026-04-01&to=2026-04-30&page=2");
    await screen.findByText("Groceries");
    const url = new URL(fetchMock.mock.calls.map(([r]) => r.url).find((u) => u.includes("/transactions")) ?? "");
    expect(Object.fromEntries(url.searchParams)).toEqual({ account_id: "1", from: "2026-04-01", to: "2026-04-30", limit: "50", offset: "50" });
    expect(screen.getByText("51 to 100 of 120")).toBeInTheDocument();
  });

  it("applies a date filter and goes back to the first page", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, [account()]),
      "GET /api/finance/transactions": page([item()], 120),
    });
    renderAt("/finance/transactions?page=2");
    await screen.findByText("Groceries");

    await userEvent.type(screen.getByLabelText("From"), "2026-04-01");

    await vi.waitFor(() => {
      const urls = fetchMock.mock.calls.map(([r]) => r.url).filter((u) => u.includes("/transactions"));
      expect(urls.at(-1)).toMatch(/from=2026-04-01/);
      expect(urls.at(-1)).toMatch(/offset=0/);
    });
  });

  it("pages forward with Next", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, []),
      "GET /api/finance/transactions": page([item()], 120),
    });
    renderAt("/finance/transactions");
    await userEvent.click(await screen.findByRole("button", { name: "Next" }));
    await vi.waitFor(() => {
      expect(fetchMock.mock.calls.some(([r]) => r.url.includes("offset=50"))).toBe(true);
    });
  });

  it("offers to clear filters when none match, and says plainly when there are none at all", async () => {
    stubApi({ "GET /api/finance/accounts": () => jsonResponse(200, []), "GET /api/finance/transactions": page([]) });
    renderAt("/finance/transactions?from=2026-04-01");
    expect(await screen.findByText("No Transactions match these filters")).toBeInTheDocument();
  });

  it("says there are no Transactions yet", async () => {
    stubApi({ "GET /api/finance/accounts": () => jsonResponse(200, []), "GET /api/finance/transactions": page([]) });
    renderAt("/finance/transactions");
    expect(await screen.findByText("No Transactions yet")).toBeInTheDocument();
  });

  it("explains a failed load and retries", async () => {
    stubApi({ "GET /api/finance/accounts": () => jsonResponse(200, []), "GET /api/finance/transactions": [() => jsonResponse(400, { status: 400, detail: "bad filter", code: "invalid_query" }), page([item()])] });
    renderAt("/finance/transactions");
    await userEvent.click(await screen.findByRole("button", { name: "Try again" }));
    expect(await screen.findByText("Groceries")).toBeInTheDocument();
  });

  it("says the Accounts could not load when the filter has none to offer", async () => {
    stubApi({ "GET /api/finance/accounts": () => jsonResponse(500, {}), "GET /api/finance/transactions": page([item()]) });
    renderAt("/finance/transactions");
    expect(await screen.findByText(/Could not load the Accounts to filter by/)).toBeInTheDocument();
    expect(screen.getByText("Groceries")).toBeInTheDocument();
  });

  it("shows that the next page is loading while the old one stays", async () => {
    let release: (response: Response) => void = () => undefined;
    stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, []),
      "GET /api/finance/transactions": [page([item()], 120), () => new Promise<Response>((resolve) => { release = resolve; })],
    });
    renderAt("/finance/transactions");
    await userEvent.click(await screen.findByRole("button", { name: "Next" }));

    expect(await screen.findByText("Loading Transactions…")).toBeInTheDocument();
    expect(screen.getByText("Groceries")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Next" })).toBeDisabled();

    release(jsonResponse(200, { total: 120, items: [item({ id: 9, description: "Second page row" })] }));
    expect(await screen.findByText("Second page row")).toBeInTheDocument();
  });
});
