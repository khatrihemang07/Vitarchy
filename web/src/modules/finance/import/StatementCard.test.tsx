import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { account, jsonResponse, preview, renderWithClient, requestsOf, section, stubApi, transaction } from "../test-utils";
import { StatementCard } from "./StatementCard";

afterEach(() => vi.unstubAllGlobals());

const accounts = () => jsonResponse(200, []);
const invalidations = { "GET /api/finance/statements": () => jsonResponse(200, []), "GET /api/finance/status": () => jsonResponse(200, {}) };

describe("StatementCard", () => {
  it("shows the file, its format, kind and period", () => {
    stubApi({ "GET /api/finance/accounts": accounts });
    renderWithClient(<StatementCard preview={preview()} />);
    expect(screen.getByText("april.pdf")).toBeInTheDocument();
    expect(screen.getByText("PDF")).toBeInTheDocument();
    expect(screen.getByText("Statement")).toBeInTheDocument();
    expect(screen.getByText("1 Apr 2026 to 30 Apr 2026")).toBeInTheDocument();
  });

  it("names the Statement password that opened the file by its label", () => {
    stubApi({ "GET /api/finance/accounts": accounts });
    renderWithClient(<StatementCard preview={preview({ encrypted: true, unlocked_with_label: "HDFC card" })} />);
    expect(screen.getByText("Opened with HDFC card")).toBeInTheDocument();
  });

  it("shows a failed Check as a warning that still allows confirming", () => {
    stubApi({ "GET /api/finance/accounts": accounts });
    renderWithClient(<StatementCard preview={preview()} />);
    const failed = screen.getByText("Totals").closest("li");
    expect(failed).toHaveAttribute("data-outcome", "failed");
    expect(within(failed as HTMLElement).getByText("Failed")).toBeInTheDocument();
    expect(screen.getByText(/A failed Check is a warning/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Confirm" })).toBeEnabled();
  });

  it("counts new and already stored rows, and dims stored ones in the Transactions table", async () => {
    stubApi({ "GET /api/finance/accounts": accounts });
    renderWithClient(<StatementCard preview={preview()} />);
    expect(screen.getByText("2 new")).toBeInTheDocument();
    expect(screen.getByText(/1 already stored/)).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: /Show 3 Transactions/ }));

    expect(screen.getByText("Stored row").closest("tr")).toHaveAttribute("data-already-stored", "true");
    expect(screen.getByText("First row").closest("tr")).toHaveAttribute("data-already-stored", "false");
    expect(screen.getByText("+₹1,234.56")).toHaveAttribute("data-direction", "credit");
  });

  it("adds 'would gain detail' to the count line and tags those rows", async () => {
    stubApi({ "GET /api/finance/accounts": accounts });
    const gaining = section({ enriched_count: 1, transactions: [transaction({ description: "Stored row", is_already_stored: true, gains_detail: true })] });
    renderWithClient(<StatementCard preview={preview({ sections: [gaining] })} />);
    expect(screen.getByText(/1 already stored, 1 would gain detail/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /Show 3 Transactions/ }));
    expect(screen.getByText("Gains detail")).toBeInTheDocument();
  });

  it("confirms with the matched Account and reports what was added", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/accounts": accounts,
      "POST /api/finance/statements/1/confirm": () => jsonResponse(200, { added: 2, already_stored: 1, enriched: 1, linked: 0 }),
      ...invalidations,
    });
    renderWithClient(<StatementCard preview={preview()} />);

    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));

    expect(await screen.findByText(/2 added, 1 already stored, 1 gained detail/)).toBeInTheDocument();
    const confirm = fetchMock.mock.calls.map(([r]) => r).find((r) => r.url.endsWith("/confirm"));
    expect(await confirm?.json()).toEqual({ sections: [{ account_id: 1 }] });
  });

  it("says when the Account was matched without a printed number", () => {
    stubApi({ "GET /api/finance/accounts": accounts });
    const unnumbered = section({ matched_without_number: true, proposed: { issuer: "HDFC", kind: "savings", mask: null } });
    renderWithClient(<StatementCard preview={preview({ sections: [unnumbered] })} />);
    expect(screen.getByText("No account number printed — matched by bank and kind")).toBeInTheDocument();
  });

  it("does not mention the number when it was printed", () => {
    stubApi({ "GET /api/finance/accounts": accounts });
    renderWithClient(<StatementCard preview={preview()} />);
    expect(screen.queryByText(/No account number printed/)).not.toBeInTheDocument();
  });

  it("asks for a New Account's details and sends them", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/accounts": accounts,
      "POST /api/finance/statements/1/confirm": () => jsonResponse(200, { added: 3, already_stored: 0, linked: 0 }),
      ...invalidations,
    });
    const unmatched = section({ matched_account: null, proposed: { issuer: "SBI", kind: "loan", mask: "XX9876" } });
    renderWithClient(<StatementCard preview={preview({ sections: [unmatched] })} />);

    const name = screen.getByLabelText("Name");
    await userEvent.clear(name);
    await userEvent.type(name, "Home loan");
    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));

    await screen.findByText(/3 added/);
    const confirm = fetchMock.mock.calls.map(([r]) => r).find((r) => r.url.endsWith("/confirm"));
    expect(await confirm?.json()).toEqual({ sections: [{ new_account: { issuer: "SBI", kind: "loan", name: "Home loan" } }] });
  });

  it("blocks Confirm until a New Account has a kind", () => {
    stubApi({ "GET /api/finance/accounts": accounts });
    const unknown = section({ matched_account: null, proposed: { issuer: null, kind: null, mask: null } });
    renderWithClient(<StatementCard preview={preview({ sections: [unknown] })} />);
    expect(screen.getByRole("button", { name: "Confirm" })).toBeDisabled();
  });

  it("explains a confirm the server refused", async () => {
    stubApi({
      "GET /api/finance/accounts": accounts,
      "POST /api/finance/statements/1/confirm": () => jsonResponse(409, { status: 409, detail: "x", code: "confirm_in_progress" }),
      ...invalidations,
    });
    renderWithClient(<StatementCard preview={preview()} />);
    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(/Another confirm is running/);
  });

  it("removes the card when the Statement is discarded", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/accounts": accounts,
      "DELETE /api/finance/statements/1": () => new Response(null, { status: 204 }),
      ...invalidations,
    });
    renderWithClient(<StatementCard preview={preview()} />);
    await userEvent.click(screen.getByRole("button", { name: "Discard" }));
    await vi.waitFor(() => { expect(screen.queryByText("april.pdf")).not.toBeInTheDocument(); });
    expect(requestsOf(fetchMock)).toContain("DELETE /api/finance/statements/1");
  });

  it("summarises a Payment history: its records and how many link now", () => {
    stubApi({ "GET /api/finance/accounts": accounts });
    const history = preview({ kind: "payment_history", sections: [], payment_history: { record_count: 195, would_link_now: 39 } });
    renderWithClient(<StatementCard preview={history} />);
    expect(screen.getByText("195 Payment records")).toBeInTheDocument();
    expect(screen.getByText(/39 link to a Transaction now/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Confirm" })).toBeEnabled();
  });

  it("explains a locked file, links to the passwords, and reads it again", async () => {
    stubApi({
      "GET /api/finance/accounts": accounts,
      "POST /api/finance/statements/1/read-again": () => jsonResponse(200, { statement: preview(), sections: [] }),
      ...invalidations,
    });
    renderWithClient(<StatementCard preview={preview({ outcome: "locked", sections: [], encrypted: true })} />);
    expect(screen.getByText("No saved password opened this file")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Add a Statement password" })).toHaveAttribute("href", "#statement-passwords");
    expect(screen.queryByRole("button", { name: "Confirm" })).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Try again" }));

    expect(await screen.findByText("Account")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Confirm" })).toBeInTheDocument();
  });

  it.each([
    ["no_table", "No table of Transactions was found"],
    ["unsupported", "This type of file is not supported"],
  ])("explains a %s file in plain words", (outcome, title) => {
    stubApi({ "GET /api/finance/accounts": accounts });
    renderWithClient(<StatementCard preview={preview({ outcome, sections: [] })} />);
    expect(screen.getByText(title)).toBeInTheDocument();
  });
});

describe("StatementCard after another Statement was confirmed", () => {
  const unmatched = () => preview({ sections: [section({ matched_account: null })] });
  const matched = () => preview({ sections: [section({ matched_account: { id: 7, name: "Existing card" } })] });

  it("switches an untouched New Account to the Account that now matches", async () => {
    stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, [account({ id: 7, name: "Existing card" })]),
      "GET /api/finance/statements/1": () => jsonResponse(200, matched()),
    });
    renderWithClient(<StatementCard preview={unmatched()} />);

    await waitFor(() => { expect(screen.getByRole("combobox", { name: "Account" })).toHaveTextContent("Existing card"); });
    expect(screen.queryByLabelText("Name")).not.toBeInTheDocument();
  });

  it("confirms the refreshed match, not the stale New Account", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, [account({ id: 7, name: "Existing card" })]),
      "GET /api/finance/statements/1": () => jsonResponse(200, matched()),
      "POST /api/finance/statements/1/confirm": () => jsonResponse(200, { added: 0, already_stored: 0, enriched: 0, linked: 0 }),
      ...invalidations,
    });
    renderWithClient(<StatementCard preview={unmatched()} />);
    await screen.findByText("Existing card");

    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));

    await screen.findByText("Confirmed.");
    const confirm = fetchMock.mock.calls.map(([r]) => r).find((r) => r.url.endsWith("/confirm"));
    expect(await confirm?.json()).toEqual({ sections: [{ account_id: 7 }] });
  });
});
