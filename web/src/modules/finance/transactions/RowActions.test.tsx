import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { TransactionItem } from "@/api/finance";
import { jsonResponse, pendingForever, renderWithClient, requestsOf, stubApi } from "../test-utils";
import { RowActions } from "./RowActions";

afterEach(() => vi.unstubAllGlobals());

const names = new Map([[1, "HDFC Savings"], [2, "SBI Card"], [3, "ICICI Card"]]);
const row = (over: Partial<TransactionItem> = {}): TransactionItem =>
  ({ id: 10, account_id: 1, date: "2026-07-05", description: "Card bill", amount_paise: 1_000_000, direction: "debit", no_longer_found: false, printing_count: 1, ...over });
const candidate = (id: number, account: number, amount: number, direction = "credit") => ({
  id, account_id: account, date: "2026-07-06", description: `Receipt ${id.toString()}`, amount_paise: amount, direction,
});
async function openDialog() {
  await userEvent.click(screen.getByRole("button", { name: "Actions for this Transaction" }));
  await userEvent.click(await screen.findByRole("button", { name: "Link as Transfer…" }));
}

const CANDIDATES = "GET /api/finance/transactions/10/transfer-candidates";

describe("RowActions", () => {
  it("shows only an actions button until it is opened, and the menu offers Link as Transfer…", async () => {
    stubApi({});
    renderWithClient(<RowActions item={row()} names={names} />);
    expect(screen.queryByRole("button", { name: "Link as Transfer…" })).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Actions for this Transaction" }));

    expect(await screen.findByRole("button", { name: "Link as Transfer…" })).toBeInTheDocument();
  });

  it("opens a dialog and lists what the row could be linked to, with Link off until something is chosen", async () => {
    stubApi({ [CANDIDATES]: () => jsonResponse(200, [candidate(20, 2, 400_000), candidate(21, 3, 600_000)]) });
    renderWithClient(<RowActions item={row()} names={names} />);

    await openDialog();

    expect(await screen.findByText("Receipt 20")).toBeInTheDocument();
    expect(screen.getByText(/SBI Card · ₹4,000.00/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Link" })).toBeDisabled();
  });

  it("links several chosen Money in rows to the Money out row and shows what they add up to", async () => {
    const fetchMock = stubApi({
      [CANDIDATES]: () => jsonResponse(200, [candidate(20, 2, 400_000), candidate(21, 3, 600_000)]),
      "POST /api/finance/transfers": () => jsonResponse(201, { debit_id: 10, credit_ids: [20, 21], transfer: { id: 5, rung: "manual", other_accounts: [2, 3], partial: false, remaining_paise: 0, extra_paise: 0 } }),
    });
    renderWithClient(<RowActions item={row()} names={names} />);
    await openDialog();
    await userEvent.click(await screen.findByRole("checkbox", { name: /Receipt 20/ }));
    await userEvent.click(screen.getByRole("checkbox", { name: /Receipt 21/ }));
    expect(screen.getByText("Chosen: ₹10,000.00 of ₹10,000.00")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Link" }));

    await vi.waitFor(() => { expect(requestsOf(fetchMock)).toContain("POST /api/finance/transfers"); });
    const post = fetchMock.mock.calls.map(([r]) => r).find((r) => r.method === "POST");
    expect(await post?.json()).toEqual({ debit_id: 10, credit_ids: [20, 21] });
    await vi.waitFor(() => { expect(screen.queryByRole("dialog")).not.toBeInTheDocument(); });
  });

  it("lets a Money in row choose just one Money out row", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/transactions/30/transfer-candidates": () => jsonResponse(200, [candidate(10, 1, 1_000_000, "debit"), candidate(11, 1, 900_000, "debit")]),
      "POST /api/finance/transfers": () => jsonResponse(201, { debit_id: 11, credit_ids: [30], transfer: { id: 5, rung: "manual", other_accounts: [3], partial: true, remaining_paise: 1, extra_paise: 0 } }),
    });
    renderWithClient(<RowActions item={row({ id: 30, account_id: 3, direction: "credit" })} names={names} />);
    await openDialog();
    await userEvent.click(await screen.findByRole("radio", { name: /Receipt 10/ }));
    await userEvent.click(screen.getByRole("radio", { name: /Receipt 11/ }));
    expect(screen.getAllByRole("radio").filter((r) => r.getAttribute("aria-checked") === "true")).toHaveLength(1);

    await userEvent.click(screen.getByRole("button", { name: "Link" }));

    await vi.waitFor(() => { expect(requestsOf(fetchMock)).toContain("POST /api/finance/transfers"); });
    const post = fetchMock.mock.calls.map(([r]) => r).find((r) => r.method === "POST");
    expect(await post?.json()).toEqual({ debit_id: 11, credit_ids: [30] });
  });

  it("says so when nothing nearby fits", async () => {
    stubApi({ [CANDIDATES]: () => jsonResponse(200, []) });
    renderWithClient(<RowActions item={row()} names={names} />);
    await openDialog();
    expect(await screen.findByText("Nothing nearby fits")).toBeInTheDocument();
  });

  it("shows a skeleton while loading and the server's reason when linking is refused", async () => {
    stubApi({ [CANDIDATES]: pendingForever });
    renderWithClient(<RowActions item={row()} names={names} />);
    await openDialog();
    expect(await screen.findByLabelText("Loading candidates")).toBeInTheDocument();
  });

  it("explains a refused link and keeps the dialog open", async () => {
    stubApi({
      [CANDIDATES]: () => jsonResponse(200, [candidate(20, 2, 400_000)]),
      "POST /api/finance/transfers": () => jsonResponse(409, { code: "transfer_already_linked", detail: "One of these Transactions is already in a Transfer. Unlink that first." }),
    });
    renderWithClient(<RowActions item={row()} names={names} />);
    await openDialog();
    await userEvent.click(await screen.findByRole("checkbox", { name: /Receipt 20/ }));
    await userEvent.click(screen.getByRole("button", { name: "Link" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("already in a Transfer");
    expect(screen.getByRole("dialog")).toBeInTheDocument();
  });
});
