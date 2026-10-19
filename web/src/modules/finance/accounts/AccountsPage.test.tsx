import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { account, jsonResponse, pendingForever, preview, renderWithClient, SETTLED_NOTICES, stubApi } from "../test-utils";
import { AccountsPage } from "./AccountsPage";

afterEach(() => vi.unstubAllGlobals());

const confirmed = {
  id: 3, file_name: "march.pdf", format: "pdf", kind: "statement", outcome: "read", status: "confirmed",
  uploaded_at: "2026-05-01T10:00:00Z", confirmed_at: "2026-05-02T10:00:00Z", account_count: 1, row_count: 30,
};

describe("AccountsPage", () => {
  it("shows only loading notices while Accounts and Statements are pending", () => {
    stubApi({ "GET /api/finance/accounts": pendingForever, "GET /api/finance/statements": pendingForever });
    const { container } = renderWithClient(<AccountsPage />);
    expect(screen.getAllByText("Loading Accounts").length).toBeGreaterThan(0);
    expect(container.querySelectorAll(SETTLED_NOTICES)).toHaveLength(0);
  });

  it("shows each Account's issuer, kind, number and figures", async () => {
    stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, [account()]),
      "GET /api/finance/statements": () => jsonResponse(200, []),
    });
    renderWithClient(<AccountsPage />);
    expect(await screen.findByText("HDFC Savings")).toBeInTheDocument();
    expect(screen.getByText("Savings account")).toBeInTheDocument();
    expect(screen.getByText("XXXX1234")).toBeInTheDocument();
    expect(screen.getByText("₹98,765.00")).toBeInTheDocument();
    expect(screen.getByText("12 Transactions")).toBeInTheDocument();
    expect(screen.getByText("1 Jan 2026 to 30 Apr 2026")).toBeInTheDocument();
    expect(screen.getByText("2 Statements")).toBeInTheDocument();
  });

  it("says so when there are no Accounts or Statements", async () => {
    stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, []),
      "GET /api/finance/statements": () => jsonResponse(200, []),
    });
    renderWithClient(<AccountsPage />);
    expect(await screen.findByText("No Accounts yet")).toBeInTheDocument();
    expect(await screen.findByText("No confirmed Statements yet.")).toBeInTheDocument();
  });

  it("explains Accounts that failed to load", async () => {
    stubApi({
      "GET /api/finance/accounts": () => jsonResponse(500, {}),
      "GET /api/finance/statements": () => jsonResponse(200, []),
    });
    renderWithClient(<AccountsPage />);
    expect(await screen.findAllByRole("alert")).not.toHaveLength(0);
  });

  it("renames an Account in place", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/accounts": [() => jsonResponse(200, [account()]), () => jsonResponse(200, [account({ name: "Salary" })])],
      "PATCH /api/finance/accounts/1": () => jsonResponse(200, account({ name: "Salary" })),
      "GET /api/finance/statements": () => jsonResponse(200, []),
    });
    renderWithClient(<AccountsPage />);

    await userEvent.click(await screen.findByRole("button", { name: "Rename HDFC Savings" }));
    const field = screen.getByLabelText("Account name");
    await userEvent.clear(field);
    await userEvent.type(field, "Salary{Enter}");

    expect(await screen.findByRole("heading", { name: "Salary" })).toBeInTheDocument();
    const patch = fetchMock.mock.calls.map(([r]) => r).find((r) => r.method === "PATCH");
    expect(await patch?.json()).toEqual({ name: "Salary" });
  });

  it("cancels a rename with Escape", async () => {
    stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, [account()]),
      "GET /api/finance/statements": () => jsonResponse(200, []),
    });
    renderWithClient(<AccountsPage />);
    await userEvent.click(await screen.findByRole("button", { name: "Rename HDFC Savings" }));
    await userEvent.type(screen.getByLabelText("Account name"), "x{Escape}");
    expect(screen.getByRole("heading", { name: "HDFC Savings" })).toBeInTheDocument();
  });

  it("previews a Read again, then applies it and reports the counts", async () => {
    stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, [account()]),
      "GET /api/finance/statements": () => jsonResponse(200, [confirmed]),
      "POST /api/finance/statements/3/read-again": () =>
        jsonResponse(200, {
          statement: preview({ id: 3, status: "confirmed" }),
          sections: [{ position: 0, add_count: 1, unchanged_count: 28, no_longer_found_count: 1, added: [
            { date: "2026-03-05", description: "Newly found row", amount_paise: 1000, direction: "debit", is_already_stored: false },
          ], new_printings: [] }],
        }),
      "POST /api/finance/statements/3/read-again/confirm": () => jsonResponse(200, { added: 1, no_longer_found: 1, unchanged: 28 }),
    });
    renderWithClient(<AccountsPage />);

    await userEvent.click(await screen.findByRole("button", { name: "Read again" }));

    expect(await screen.findByText(/1 to add, 28 unchanged, 1 no longer found/)).toBeInTheDocument();
    expect(screen.getByText("Newly found row")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Apply changes" }));
    expect(await screen.findByText(/1 added, 28 unchanged, 1 no longer found/)).toBeInTheDocument();
  });

  it("explains a Read again that finds different Accounts", async () => {
    stubApi({
      "GET /api/finance/accounts": () => jsonResponse(200, []),
      "GET /api/finance/statements": () => jsonResponse(200, [confirmed]),
      "POST /api/finance/statements/3/read-again": () => jsonResponse(409, { status: 409, detail: "x", code: "statement_sections_changed" }),
    });
    renderWithClient(<AccountsPage />);
    await userEvent.click(await screen.findByRole("button", { name: "Read again" }));
    expect(await screen.findByText(/different Accounts/)).toBeInTheDocument();
  });
});
