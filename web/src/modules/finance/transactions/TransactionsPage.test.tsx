import { createMemoryHistory, createRootRoute, createRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TooltipProvider } from "@/design";
import { periodFor, todayOf } from "@/lib/period/period";
import { account, jsonResponse, overview, pendingForever, SETTLED_NOTICES, stubApi } from "../test-utils";
import { parseTransactionsSearch, transactionsParams } from "./transactionsSearch";
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
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <TooltipProvider>
        <RouterProvider router={router} />
      </TooltipProvider>
    </QueryClientProvider>,
  );
  return router;
}

const item = (over: Record<string, unknown> = {}) => ({
  id: 1, account_id: 1, date: "2026-04-02", description: "Groceries", amount_paise: 12_345_678, direction: "debit",
  balance_paise: 50_000_000, no_longer_found: false, printing_count: 1, ...over,
});
const page = (items: unknown[], total = items.length) => () => jsonResponse(200, { total, items });
const accounts = (list = [account()]) => () => jsonResponse(200, list);
const overviewReply = () => jsonResponse(200, overview());
const APRIL = "kind=month&date=2026-04-10";

/** The fetches a stub saw whose path ends with `suffix`, as their query strings. */
function queriesOf(fetchMock: ReturnType<typeof stubApi>, suffix: string): URLSearchParams[] {
  return fetchMock.mock.calls.map(([r]) => new URL(r.url)).filter((u) => u.pathname.endsWith(suffix)).map((u) => u.searchParams);
}

function stubs(over: Record<string, Parameters<typeof stubApi>[0][string]> = {}) {
  return stubApi({
    "GET /api/finance/accounts": accounts(),
    "GET /api/finance/transactions": page([item()]),
    "GET /api/finance/overview": overviewReply,
    ...over,
  });
}

describe("TransactionsPage", () => {
  it("shows only a loading notice, never an empty or failure one, while the queries are pending", async () => {
    stubApi({ "GET /api/finance/accounts": pendingForever, "GET /api/finance/transactions": pendingForever, "GET /api/finance/overview": pendingForever });
    renderAt("/finance/transactions");
    await screen.findByText("Loading Transactions");
    expect(document.querySelectorAll(SETTLED_NOTICES)).toHaveLength(0);
  });

  it("shows amounts in INR with Indian grouping and the Account's name", async () => {
    stubs();
    renderAt("/finance/transactions");
    expect(await screen.findByText("−₹1,23,456.78")).toHaveAttribute("data-direction", "debit");
    expect(screen.getAllByText("₹5,00,000.00").length).toBeGreaterThan(0);
    expect(await screen.findByText("HDFC Savings", { selector: "span" })).toBeInTheDocument();
  });

  it("marks a Transfer row with its other Account and offers linking on a row that is not one", async () => {
    stubs({
      "GET /api/finance/accounts": accounts([account(), account({ id: 2, name: "SBI Card", kind: "credit_card" })]),
      "GET /api/finance/transactions": page([
        item({ transfer: { id: 4, rung: "bill_payment", other_accounts: [2], partial: false, remaining_paise: 0, extra_paise: 0 } }),
        item({ id: 2, description: "Cafe" }),
      ]),
    });
    renderAt("/finance/transactions");
    expect(await screen.findByRole("button", { name: "Transfer → SBI Card, details" })).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Actions for this Transaction" })).toHaveLength(1);
  });

  it("flags rows no longer found", async () => {
    stubs({ "GET /api/finance/transactions": page([item(), item({ id: 2, description: "Old row", no_longer_found: true })]) });
    renderAt("/finance/transactions");
    expect(await screen.findByText("Old row")).toBeInTheDocument();
    expect(screen.getByText("No longer found")).toBeInTheDocument();
  });

  it("asks for this month when the address names no Period", async () => {
    const fetchMock = stubs();
    renderAt("/finance/transactions");
    await screen.findByText("Groceries");
    const month = periodFor("month", todayOf(new Date()), "monday");
    expect(Object.fromEntries(queriesOf(fetchMock, "/transactions")[0] ?? [])).toEqual({ ...month, limit: "50", offset: "0" });
    expect(Object.fromEntries(queriesOf(fetchMock, "/overview")[0] ?? [])).toEqual(month);
  });

  it("asks the server only for the filters in the address, and the Overview for the same Accounts and search", async () => {
    const fetchMock = stubs({ "GET /api/finance/transactions": page([item()], 120) });
    renderAt(`/finance/transactions?${APRIL}&account=1&account=2&q=rent&direction=out&sort=amount_asc&page=2`);
    await screen.findByText("Groceries");
    const asked = queriesOf(fetchMock, "/transactions")[0];
    expect(asked?.getAll("account_id")).toEqual(["1", "2"]);
    expect(Object.fromEntries(asked ?? [])).toMatchObject({ from: "2026-04-01", to: "2026-04-30", q: "rent", direction: "out", sort: "amount_asc", limit: "50", offset: "50" });
    const overviewAsked = queriesOf(fetchMock, "/overview")[0];
    expect(overviewAsked?.getAll("account_id")).toEqual(["1", "2"]);
    expect(Object.fromEntries(overviewAsked ?? [])).toMatchObject({ from: "2026-04-01", to: "2026-04-30", q: "rent" });
    expect(overviewAsked?.has("direction")).toBe(false);
    expect(screen.getByText("51 to 100 of 120")).toBeInTheDocument();
  });

  it("shows the Overview: four figures against the period just before, and the note about Transfers", async () => {
    stubs();
    renderAt(`/finance/transactions?${APRIL}`);
    const strip = await screen.findByRole("region", { name: "Overview" });
    for (const label of ["Money in", "Money out", "Net", "Transfers"]) expect(within(strip).getAllByText(label).length).toBeGreaterThan(0);
    expect(within(strip).getAllByText("↑ ₹1,200.00")).toHaveLength(2);
    expect(within(strip).getByText("↓ ₹1,500.00")).toBeInTheDocument();
    expect(within(strip).getByText("Transfers between your Accounts are left out of Money in and Money out.")).toBeInTheDocument();
  });

  it("lets the table show when the Overview cannot load", async () => {
    stubs({ "GET /api/finance/overview": () => jsonResponse(500, {}) });
    renderAt(`/finance/transactions?${APRIL}`);
    expect(await screen.findByText("Groceries")).toBeInTheDocument();
    expect(await screen.findByRole("button", { name: "Try again" })).toBeInTheDocument();
  });

  it("moves to another Period and back to the first page", async () => {
    const fetchMock = stubs({ "GET /api/finance/transactions": page([item()], 120) });
    const router = renderAt(`/finance/transactions?${APRIL}&page=2`);
    await screen.findByText("Groceries");
    await userEvent.click(screen.getByRole("button", { name: "Next month" }));
    await vi.waitFor(() => {
      expect(queriesOf(fetchMock, "/transactions").at(-1)?.get("from")).toBe("2026-05-01");
    });
    expect(queriesOf(fetchMock, "/transactions").at(-1)?.get("offset")).toBe("0");
    expect(router.state.location.search).toMatchObject({ date: "2026-05-01" });
    expect(router.state.location.search).not.toHaveProperty("page");
  });

  it("searches after a pause in typing and writes the search to the address", async () => {
    const fetchMock = stubs();
    const router = renderAt(`/finance/transactions?${APRIL}`);
    await screen.findByText("Groceries");
    await userEvent.type(screen.getByRole("searchbox", { name: "Search descriptions" }), "rent");
    await vi.waitFor(() => {
      expect(queriesOf(fetchMock, "/transactions").at(-1)?.get("q")).toBe("rent");
    });
    expect(queriesOf(fetchMock, "/transactions").filter((q) => q.has("q"))).toHaveLength(1);
    expect(router.state.location.search).toMatchObject({ q: "rent" });
  });

  it("filters by direction", async () => {
    const fetchMock = stubs();
    renderAt(`/finance/transactions?${APRIL}`);
    await screen.findByText("Groceries");
    await userEvent.click(screen.getByRole("radio", { name: "Transfers" }));
    await vi.waitFor(() => {
      expect(queriesOf(fetchMock, "/transactions").at(-1)?.get("direction")).toBe("transfer");
    });
    expect(queriesOf(fetchMock, "/overview").at(-1)?.has("direction")).toBe(false);
  });

  it("filters by several Accounts and leaves hidden ones out of the choices", async () => {
    const fetchMock = stubs({ "GET /api/finance/accounts": accounts([account(), account({ id: 2, name: "SBI Card" }), account({ id: 3, name: "Old loan", hidden: true })]) });
    renderAt(`/finance/transactions?${APRIL}`);
    await screen.findByText("Groceries");
    await userEvent.click(screen.getByRole("button", { name: "Account filter" }));
    expect(screen.queryByText("Old loan")).not.toBeInTheDocument();
    await userEvent.click(await screen.findByText("SBI Card"));
    await vi.waitFor(() => {
      expect(queriesOf(fetchMock, "/transactions").at(-1)?.getAll("account_id")).toEqual(["2"]);
    });
  });

  it("sorts by a column header, flips it on a second click, and says so with aria-sort", async () => {
    const fetchMock = stubs();
    renderAt(`/finance/transactions?${APRIL}`);
    await screen.findByText("Groceries");
    expect(screen.getByRole("columnheader", { name: "Date" })).toHaveAttribute("aria-sort", "descending");

    await userEvent.click(screen.getByRole("button", { name: "Amount" }));
    await vi.waitFor(() => {
      expect(queriesOf(fetchMock, "/transactions").at(-1)?.get("sort")).toBe("amount_desc");
    });
    expect(screen.getByRole("columnheader", { name: "Amount" })).toHaveAttribute("aria-sort", "descending");
    expect(screen.getByRole("columnheader", { name: "Date" })).toHaveAttribute("aria-sort", "none");

    await userEvent.click(screen.getByRole("button", { name: "Amount" }));
    await vi.waitFor(() => {
      expect(queriesOf(fetchMock, "/transactions").at(-1)?.get("sort")).toBe("amount_asc");
    });
    expect(screen.getByRole("columnheader", { name: "Amount" })).toHaveAttribute("aria-sort", "ascending");
  });

  it("clears the Accounts, the search and the direction in one click but keeps the Period", async () => {
    const fetchMock = stubs();
    renderAt(`/finance/transactions?${APRIL}&account=1&q=rent&direction=in`);
    await screen.findByText("Groceries");
    await userEvent.click(screen.getByRole("button", { name: "Clear filters" }));
    await vi.waitFor(() => {
      const last = queriesOf(fetchMock, "/transactions").at(-1);
      expect(last?.has("q") || last?.has("account_id") || last?.has("direction")).toBe(false);
      expect(last?.get("from")).toBe("2026-04-01");
    });
    expect(screen.getByRole("searchbox")).toHaveValue("");
  });

  it("pages forward with Next", async () => {
    const fetchMock = stubs({ "GET /api/finance/transactions": page([item()], 120) });
    renderAt(`/finance/transactions?${APRIL}`);
    await userEvent.click(await screen.findByRole("button", { name: "Next" }));
    await vi.waitFor(() => {
      expect(fetchMock.mock.calls.some(([r]) => r.url.includes("offset=50"))).toBe(true);
    });
  });

  it("says no Transactions match when filters are on, and offers to clear them", async () => {
    stubs({ "GET /api/finance/transactions": page([]) });
    renderAt(`/finance/transactions?${APRIL}&q=zzz`);
    expect(await screen.findByText("No Transactions match these filters")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Clear filters" })).toHaveLength(2);
  });

  it("says plainly that the Period is empty, or that there are no Transactions at all", async () => {
    stubs({ "GET /api/finance/transactions": page([]) });
    renderAt(`/finance/transactions?${APRIL}`);
    expect(await screen.findByText("No Transactions in this Period")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Clear filters" })).not.toBeInTheDocument();
  });

  it("says there are no Transactions yet", async () => {
    stubs({ "GET /api/finance/accounts": accounts([account({ transaction_count: 0 })]), "GET /api/finance/transactions": page([]) });
    renderAt("/finance/transactions");
    expect(await screen.findByText("No Transactions yet")).toBeInTheDocument();
  });

  it("explains a failed load and retries", async () => {
    stubs({ "GET /api/finance/transactions": [() => jsonResponse(400, { status: 400, detail: "bad filter", code: "invalid_query" }), page([item()])] });
    renderAt("/finance/transactions");
    await userEvent.click(await screen.findByRole("button", { name: "Try again" }));
    expect(await screen.findByText("Groceries")).toBeInTheDocument();
  });

  it("says the Accounts could not load when the filter has none to offer", async () => {
    stubs({ "GET /api/finance/accounts": () => jsonResponse(500, {}) });
    renderAt("/finance/transactions");
    expect(await screen.findByText(/Could not load the Accounts to filter by/)).toBeInTheDocument();
  });

  it("shows that the next page is loading while the old one stays", async () => {
    let release: (response: Response) => void = () => undefined;
    stubs({ "GET /api/finance/transactions": [page([item()], 120), () => new Promise<Response>((resolve) => { release = resolve; })] });
    renderAt("/finance/transactions");
    await userEvent.click(await screen.findByRole("button", { name: "Next" }));

    expect(await screen.findByText("Loading Transactions…")).toBeInTheDocument();
    expect(screen.getByText("Groceries")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Next" })).toBeDisabled();

    release(jsonResponse(200, { total: 120, items: [item({ id: 9, description: "Second page row" })] }));
    expect(await screen.findByText("Second page row")).toBeInTheDocument();
  });

  it("keeps the old rows, dimmed, while a changed filter loads", async () => {
    stubs({ "GET /api/finance/transactions": [page([item()]), () => new Promise<Response>(() => undefined)] });
    renderAt(`/finance/transactions?${APRIL}`);
    await screen.findByText("Groceries");
    await userEvent.click(screen.getByRole("radio", { name: "Money out" }));
    await vi.waitFor(() => {
      expect(document.querySelector("[aria-busy=true]")).toHaveClass("opacity-60", "transition-opacity");
    });
    expect(screen.getByText("Groceries")).toBeInTheDocument();
  });

  it("keeps the fade on the table while it is not dimmed, so the dimming eases back", async () => {
    stubs();
    renderAt("/finance/transactions");
    await screen.findByText("Groceries");
    const tables = [...document.querySelectorAll("[aria-busy]")];
    expect(tables.length).toBeGreaterThan(0);
    for (const table of tables) {
      expect(table).toHaveClass("transition-opacity");
      expect(table).not.toHaveClass("opacity-60");
    }
  });

  it("holds the rows back until the Account names are in, instead of showing a placeholder name", async () => {
    stubs({ "GET /api/finance/accounts": pendingForever });
    renderAt("/finance/transactions");
    expect(await screen.findByText("Loading Transactions")).toBeInTheDocument();
    expect(screen.queryByText("Groceries")).not.toBeInTheDocument();
    expect(screen.queryByText("Account", { selector: "span" })).not.toBeInTheDocument();
  });

  it("shows no skeleton for a load that finishes inside the loading delay", async () => {
    document.documentElement.style.setProperty("--duration-loading-delay", "250ms");
    try {
      stubs();
      renderAt("/finance/transactions");
      expect(screen.queryByText("Loading Transactions")).not.toBeInTheDocument();
      expect(await screen.findByText("Groceries")).toBeInTheDocument();
      expect(screen.queryByText("Loading Transactions")).not.toBeInTheDocument();
    } finally {
      document.documentElement.style.removeProperty("--duration-loading-delay");
    }
  });
});
