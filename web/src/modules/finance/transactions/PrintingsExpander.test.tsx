import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { TransactionItem } from "@/api/finance";
import { jsonResponse, pendingForever, renderWithClient, requestsOf, stubApi } from "../test-utils";
import { TransactionRow } from "./TransactionRow";

afterEach(() => vi.unstubAllGlobals());

const item = (over: Partial<TransactionItem> = {}): TransactionItem => ({
  id: 7, account_id: 1, date: "2026-04-02", description: "UPI payment to a shop, full text", amount_paise: 50_000,
  direction: "debit", balance_paise: 100_000, no_longer_found: false, printing_count: 2, ...over,
});
const printing = (over: Record<string, unknown> = {}) => ({
  id: 1, statement_id: 1, file_name: "april.pdf", date: "2026-04-02", description: "UPI payment to a sh",
  balance_paise: 100_000, reference: null, gone: false, ...over,
});

function renderRow(row: TransactionItem) {
  return renderWithClient(<ul><TransactionRow item={row} names={new Map()} accountName="HDFC Savings" /></ul>);
}
const PATH = "GET /api/finance/transactions/7/printings";

describe("Printings expander", () => {
  it("shows no expander for a Transaction with one Printing", () => {
    renderRow(item({ printing_count: 1 }));
    expect(screen.queryByRole("button", { name: /Printed in/ })).toBeNull();
  });

  it("fetches nothing until opened, then lists each Printing with its gone tag", async () => {
    const fetchMock = stubApi({
      [PATH]: () => jsonResponse(200, [printing(), printing({ id: 2, file_name: "april.xlsx", reference: "UTR123", gone: true })]),
    });
    renderRow(item());
    const toggle = screen.getByRole("button", { name: "Printed in 2 Statements" });
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(requestsOf(fetchMock)).toEqual([]);
    await userEvent.click(toggle);
    expect(await screen.findByText("april.xlsx")).toBeInTheDocument();
    expect(screen.getByText("april.pdf")).toBeInTheDocument();
    expect(screen.getByText("Ref UTR123")).toBeInTheDocument();
    expect(screen.getAllByText("No longer found")).toHaveLength(1);
    expect(toggle).toHaveAttribute("aria-expanded", "true");
  });

  it("shows the time next to the date, for a Printing from a payment app too", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, [printing({ file_name: "phonepe.csv", time: "11:20:05", date: "2026-08-18" })]) });
    renderRow(item({ date: "2026-08-18", time: "11:20:05" }));
    expect(screen.getByText("18 Aug 2026 · 11:20")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /Printed in 2/ }));
    expect(await screen.findByText("phonepe.csv")).toBeInTheDocument();
    expect(screen.getAllByText("18 Aug 2026 · 11:20")).toHaveLength(2);
  });

  it("opens from the keyboard", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, [printing()]) });
    renderRow(item());
    screen.getByRole("button", { name: /Printed in 2/ }).focus();
    await userEvent.keyboard("{Enter}");
    expect(await screen.findByText("april.pdf")).toBeInTheDocument();
  });

  it("shows only a loading notice while pending", async () => {
    stubApi({ [PATH]: pendingForever });
    renderRow(item());
    await userEvent.click(screen.getByRole("button", { name: /Printed in/ }));
    expect(await screen.findByText("Loading Printings…")).toBeInTheDocument();
    expect(document.querySelectorAll('[data-kind="failure"], [data-kind="empty"]')).toHaveLength(0);
  });

  it("explains a failure and retries", async () => {
    const fetchMock = stubApi({
      [PATH]: [() => jsonResponse(404, { detail: "No such Transaction.", code: "transaction_not_found" }), () => jsonResponse(200, [printing()])],
    });
    renderRow(item());
    await userEvent.click(screen.getByRole("button", { name: /Printed in/ }));
    expect(await screen.findByText(/Could not load the Printings/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByText("april.pdf")).toBeInTheDocument();
    expect(requestsOf(fetchMock)).toHaveLength(2);
  });

  it("says so when no Printings come back", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, []) });
    renderRow(item());
    await userEvent.click(screen.getByRole("button", { name: /Printed in/ }));
    expect(await screen.findByText("No Printings are stored for this Transaction.")).toBeInTheDocument();
  });
});
