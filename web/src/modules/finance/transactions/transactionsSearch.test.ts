import { clearFilters, hasActiveFilters, onFirstPage, parseTransactionsSearch, transactionsParams } from "./transactionsSearch";

describe("parseTransactionsSearch", () => {
  it("defaults to this month, every Account, no search, every direction, newest first", () => {
    expect(parseTransactionsSearch({})).toEqual({ kind: "month", accounts: [], q: "", direction: "all", sort: "date_desc" });
  });

  it("keeps valid values", () => {
    const parsed = parseTransactionsSearch({
      kind: "week", date: "2026-10-16", account: [4, 7], q: "rent", direction: "transfer", sort: "amount_asc", page: "3",
    });
    expect(parsed).toEqual({ kind: "week", date: "2026-10-16", accounts: [4, 7], q: "rent", direction: "transfer", sort: "amount_asc", page: 3 });
  });

  it("reads a custom range and puts a backwards one right", () => {
    expect(parseTransactionsSearch({ kind: "custom", from: "2026-05-01", to: "2026-04-01" })).toMatchObject({ kind: "custom", from: "2026-04-01", to: "2026-05-01" });
  });

  it("accepts one Account where a list is expected, as an older address had", () => {
    expect(parseTransactionsSearch({ account: "4" }).accounts).toEqual([4]);
  });

  it("drops anything unusable", () => {
    const parsed = parseTransactionsSearch({ kind: "year", account: ["x", -2, 3, 3], direction: "up", sort: "random", page: -2, date: "2026-13-40" });
    expect(parsed).toEqual({ kind: "month", accounts: [3], q: "", direction: "all", sort: "date_desc" });
  });

  it("drops the default first page so the address stays short", () => {
    expect(parseTransactionsSearch({ page: 1 })).not.toHaveProperty("page");
  });

  it("keeps numeric-looking text as text (the router parses 2026 into a number)", () => {
    expect(parseTransactionsSearch({ q: 2026 }).q).toBe("2026");
  });
});

describe("transactionsParams", () => {
  it("writes nothing for the defaults", () => {
    expect(transactionsParams(parseTransactionsSearch({}))).toEqual({});
  });

  it("writes only what is not a default", () => {
    const search = parseTransactionsSearch({ kind: "week", account: [2], q: "rent", direction: "out", sort: "date_asc", page: 2 });
    expect(transactionsParams(search)).toEqual({ kind: "week", account: [2], q: "rent", direction: "out", sort: "date_asc", page: 2 });
  });

  it("round-trips through parse", () => {
    const search = parseTransactionsSearch({ kind: "custom", from: "2026-04-01", to: "2026-04-30", account: [1, 2], direction: "in", sort: "amount_desc" });
    expect(parseTransactionsSearch(transactionsParams(search))).toEqual(search);
  });
});

describe("filters", () => {
  it("count an Account, a search or a direction, but not the Period or the order", () => {
    expect(hasActiveFilters(parseTransactionsSearch({ kind: "week", sort: "date_asc" }))).toBe(false);
    expect(hasActiveFilters(parseTransactionsSearch({ account: 1 }))).toBe(true);
    expect(hasActiveFilters(parseTransactionsSearch({ q: "x" }))).toBe(true);
    expect(hasActiveFilters(parseTransactionsSearch({ direction: "in" }))).toBe(true);
    expect(hasActiveFilters(parseTransactionsSearch({ q: "   " }))).toBe(false);
  });

  it("clear without touching the Period or the order, and return to the first page", () => {
    const search = parseTransactionsSearch({ kind: "week", account: [1], q: "x", direction: "in", sort: "date_asc", page: 3 });
    expect(clearFilters(search)).toEqual({ kind: "week", accounts: [], q: "", direction: "all", sort: "date_asc" });
  });

  it("return to the first page", () => {
    expect(onFirstPage(parseTransactionsSearch({ page: 4 }))).not.toHaveProperty("page");
  });
});
