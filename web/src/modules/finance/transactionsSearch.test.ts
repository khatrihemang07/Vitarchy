import { pageRange, parseTransactionsSearch, toTransactionQuery, withFilter } from "./transactionsSearch";

describe("parseTransactionsSearch", () => {
  it("keeps valid values", () => {
    expect(parseTransactionsSearch({ account: "4", from: "2026-04-01", to: "2026-04-30", page: "3" })).toEqual({
      account: 4,
      from: "2026-04-01",
      to: "2026-04-30",
      page: 3,
    });
  });
  it("drops anything unusable", () => {
    expect(parseTransactionsSearch({ account: "abc", from: "2026-13-40", to: "yesterday", page: -2 })).toEqual({});
  });
  it("drops the default first page so the address stays short", () => {
    expect(parseTransactionsSearch({ page: 1 })).toEqual({});
  });
  it("puts a backwards range right", () => {
    expect(parseTransactionsSearch({ from: "2026-05-01", to: "2026-04-01" })).toEqual({ from: "2026-04-01", to: "2026-05-01" });
  });
});

describe("withFilter", () => {
  it("returns to the first page when a filter changes", () => {
    expect(withFilter({ account: 1, page: 4 }, { from: "2026-04-01" })).toEqual({ account: 1, from: "2026-04-01" });
  });
  it("clears a filter set to undefined", () => {
    expect(withFilter({ account: 1 }, { account: undefined })).toEqual({});
  });
});

describe("toTransactionQuery", () => {
  it("turns the page into limit and offset", () => {
    expect(toTransactionQuery({ account: 2, page: 3 })).toEqual({ account_id: 2, limit: 50, offset: 100 });
  });
});

describe("pageRange", () => {
  it("describes the middle, first and empty pages", () => {
    expect(pageRange(230, 2)).toMatchObject({ first: 51, last: 100, hasPrevious: true, hasNext: true, pageCount: 5 });
    expect(pageRange(30, 1)).toMatchObject({ first: 1, last: 30, hasPrevious: false, hasNext: false });
    expect(pageRange(0, 1)).toMatchObject({ first: 0, last: 0, pageCount: 1 });
  });
});
