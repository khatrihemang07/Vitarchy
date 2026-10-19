import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TooltipProvider } from "@/design";
import { account } from "../test-utils";
import { parseTransactionsSearch, type TransactionsSearch } from "./transactionsSearch";
import { TransactionsFilterBar } from "./TransactionsFilterBar";
import type { useTransactionsHandlers } from "./useTransactionsHandlers";

const handlers = () => ({
  onKind: vi.fn(), onShift: vi.fn(), onToday: vi.fn(), onCustom: vi.fn(), onAccounts: vi.fn(), onSearch: vi.fn(),
  onDirection: vi.fn(), onSort: vi.fn(), onSortColumn: vi.fn(), onClear: vi.fn(), onPage: vi.fn(),
});

function show(raw: Record<string, unknown> = {}, h = handlers()) {
  const search: TransactionsSearch = parseTransactionsSearch({ kind: "month", date: "2026-10-10", ...raw });
  render(
    <TooltipProvider>
      <TransactionsFilterBar
        search={search}
        range={{ from: "2026-10-01", to: "2026-10-31" }}
        today="2026-10-19"
        accounts={[account(), account({ id: 2, name: "SBI Card" }), account({ id: 3, name: "Old loan", hidden: true })]}
        handlers={h as unknown as ReturnType<typeof useTransactionsHandlers>}
      />
    </TooltipProvider>,
  );
  return h;
}

describe("TransactionsFilterBar", () => {
  it("shows the Period and has no All choice", () => {
    show();
    expect(screen.getByText("October 2026")).toBeInTheDocument();
    expect(screen.queryByRole("radio", { name: "All time" })).not.toBeInTheDocument();
  });

  it("offers the direction choices and reports the one picked", async () => {
    const h = show();
    expect(screen.getAllByRole("radio").map((r) => r.textContent).slice(-4)).toEqual(["All", "Money in", "Money out", "Transfers"]);
    await userEvent.click(screen.getByRole("radio", { name: "Money out" }));
    expect(h.onDirection).toHaveBeenCalledWith("out");
  });

  it("picks Accounts as numbers and never offers a hidden one", async () => {
    const h = show({ account: [1] });
    await userEvent.click(screen.getByRole("button", { name: "Account filter" }));
    expect(screen.queryByText("Old loan")).not.toBeInTheDocument();
    await userEvent.click(await screen.findByText("SBI Card"));
    expect(h.onAccounts).toHaveBeenCalledWith([1, 2]);
  });

  it("shows Clear filters only while an Account, a search or a direction narrows the list", () => {
    show({ kind: "week", sort: "date_asc" });
    expect(screen.queryByRole("button", { name: "Clear filters" })).not.toBeInTheDocument();
  });

  it("clears the filters with one button", async () => {
    const h = show({ q: "rent" });
    await userEvent.click(screen.getByRole("button", { name: "Clear filters" }));
    expect(h.onClear).toHaveBeenCalledOnce();
    expect(screen.getByRole("searchbox")).toHaveValue("rent");
  });

  it("has a sort picker for screens without the header bar", () => {
    show({ sort: "amount_desc" });
    expect(screen.getByRole("combobox", { name: "Sort by" })).toHaveTextContent("Largest first");
  });
});
