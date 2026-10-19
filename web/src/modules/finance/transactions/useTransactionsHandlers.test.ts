import { renderHook } from "@testing-library/react";
import { parseTransactionsSearch, type TransactionsSearch } from "./transactionsSearch";
import { useTransactionsHandlers } from "./useTransactionsHandlers";

function setup(raw: Record<string, unknown> = {}) {
  const update = vi.fn<(next: TransactionsSearch) => void>();
  const search = parseTransactionsSearch({ kind: "month", date: "2026-10-10", ...raw });
  const { result } = renderHook(() => useTransactionsHandlers({ search, range: { from: "2026-10-01", to: "2026-10-31" }, today: "2026-10-19", weekStart: "monday", update }));
  return { handlers: result.current, update };
}

describe("useTransactionsHandlers", () => {
  it("returns to the first page whenever a filter, the order or the Period changes", () => {
    const { handlers, update } = setup({ page: 3 });
    handlers.onAccounts([2]);
    handlers.onSearch("rent");
    handlers.onDirection("out");
    handlers.onSort("amount_asc");
    handlers.onShift(1);
    for (const [next] of update.mock.calls) expect(next).not.toHaveProperty("page");
    expect(update).toHaveBeenCalledTimes(5);
  });

  it("keeps everything else when one thing changes", () => {
    const { handlers, update } = setup({ account: [1], q: "x" });
    handlers.onDirection("in");
    expect(update).toHaveBeenCalledWith(expect.objectContaining({ accounts: [1], q: "x", direction: "in" }));
  });

  it("sorts a clicked column newest or largest first, then flips it", () => {
    const { handlers, update } = setup();
    handlers.onSortColumn("amount");
    handlers.onSortColumn("date");
    expect(update.mock.calls.map(([n]) => n.sort)).toEqual(["amount_desc", "date_asc"]);
  });

  it("pages without leaving the page's filters", () => {
    const { handlers, update } = setup({ q: "x" });
    handlers.onPage(2);
    expect(update).toHaveBeenCalledWith(expect.objectContaining({ q: "x", page: 2 }));
  });

  it("clears Accounts, search and direction but keeps the Period and the order", () => {
    const { handlers, update } = setup({ account: [1], q: "x", direction: "in", sort: "date_asc", page: 2 });
    handlers.onClear();
    expect(update).toHaveBeenCalledWith({ kind: "month", date: "2026-10-10", accounts: [], q: "", direction: "all", sort: "date_asc" });
  });
});
