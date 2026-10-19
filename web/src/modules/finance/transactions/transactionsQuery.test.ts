import { pageRange, toOverviewQuery, toTransactionQuery } from "./transactionsQuery";
import { parseTransactionsSearch } from "./transactionsSearch";

const october = { from: "2026-10-01", to: "2026-10-31" };

describe("toTransactionQuery", () => {
  it("asks for the range and the first page when nothing else is chosen", () => {
    expect(toTransactionQuery(parseTransactionsSearch({}), october)).toEqual({ ...october, limit: 50, offset: 0 });
  });

  it("carries every filter, the order and the page as limit and offset", () => {
    const search = parseTransactionsSearch({ account: [2, 5], q: " rent ", direction: "out", sort: "amount_desc", page: 3 });
    expect(toTransactionQuery(search, october)).toEqual({ ...october, account_id: [2, 5], q: "rent", direction: "out", sort: "amount_desc", limit: 50, offset: 100 });
  });
});

describe("toOverviewQuery", () => {
  it("keeps the Accounts and the search but not the direction, order or page", () => {
    const search = parseTransactionsSearch({ account: [2], q: "rent", direction: "in", sort: "amount_asc", page: 4 });
    expect(toOverviewQuery(search, october)).toEqual({ ...october, account_id: [2], q: "rent" });
  });
});

describe("pageRange", () => {
  it("describes the middle, first and empty pages", () => {
    expect(pageRange(230, 2)).toMatchObject({ first: 51, last: 100, hasPrevious: true, hasNext: true, pageCount: 5 });
    expect(pageRange(30, 1)).toMatchObject({ first: 1, last: 30, hasPrevious: false, hasNext: false });
    expect(pageRange(0, 1)).toMatchObject({ first: 0, last: 0, pageCount: 1 });
  });
});
