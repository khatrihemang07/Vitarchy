import { QueryClient } from "@tanstack/react-query";
import { jsonResponse, requestsOf, stubApi } from "../test-utils";
import { transactionQueriesFor, warmTransactions } from "./transactionsLoader";
import { parseTransactionsSearch } from "./transactionsSearch";

afterEach(() => vi.unstubAllGlobals());

const october19 = new Date(2026, 9, 19, 12);

describe("transactionQueriesFor", () => {
  it("asks for this month by default", () => {
    const { transactions, overview } = transactionQueriesFor(parseTransactionsSearch({}), "monday", october19);
    expect(overview).toEqual({ from: "2026-10-01", to: "2026-10-31" });
    expect(transactions).toMatchObject({ from: "2026-10-01", to: "2026-10-31", limit: 50, offset: 0 });
  });

  it("follows the week start for a week", () => {
    const search = parseTransactionsSearch({ kind: "week" });
    expect(transactionQueriesFor(search, "monday", october19).overview).toMatchObject({ from: "2026-10-19", to: "2026-10-25" });
    expect(transactionQueriesFor(search, "sunday", october19).overview).toMatchObject({ from: "2026-10-18", to: "2026-10-24" });
  });
});

describe("warmTransactions", () => {
  it("loads the Accounts, the page and the Overview of the URL's filters into the cache", async () => {
    const fetchMock = stubApi({
      "GET /api/settings": () => jsonResponse(200, { core: {} }),
      "GET /api/finance/accounts": () => jsonResponse(200, []),
      "GET /api/finance/transactions": () => jsonResponse(200, { total: 0, items: [] }),
      "GET /api/finance/overview": () => jsonResponse(200, {}),
    });
    await warmTransactions({ q: "rent" })(new QueryClient());
    const calls = requestsOf(fetchMock);
    expect(calls).toEqual(expect.arrayContaining(["GET /api/finance/accounts", "GET /api/finance/transactions", "GET /api/finance/overview"]));
    const overviewUrl = fetchMock.mock.calls.map(([r]) => r.url).find((u) => u.includes("/overview")) ?? "";
    expect(new URL(overviewUrl).searchParams.get("q")).toBe("rent");
  });
});
