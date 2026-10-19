import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { TransferInfo } from "@/api/finance";
import { jsonResponse, renderWithClient, requestsOf, stubApi } from "../test-utils";
import { TransferBadge } from "./TransferBadge";

afterEach(() => vi.unstubAllGlobals());

const names = new Map([[1, "HDFC Savings"], [2, "SBI Card"], [3, "ICICI Card"]]);
const transfer = (over: Partial<TransferInfo> = {}): TransferInfo => ({
  id: 9, rung: "bill_payment", other_accounts: [2], partial: false, remaining_paise: 0, extra_paise: 0, ...over,
});

describe("TransferBadge", () => {
  it("reads Transfer → and the Account on the other side", () => {
    renderWithClient(<TransferBadge transfer={transfer()} direction="debit" names={names} />);
    expect(screen.getByRole("button", { name: "Transfer → SBI Card, details" })).toHaveTextContent("Transfer → SBI Card");
  });

  it("opens to the Accounts paid, what is still on its way and how it was linked", async () => {
    renderWithClient(
      <TransferBadge transfer={transfer({ other_accounts: [2, 3], partial: true, remaining_paise: 230_648_300 })} direction="debit" names={names} />,
    );
    await userEvent.click(screen.getByRole("button", { name: /Transfer → SBI Card, ICICI Card/ }));
    expect(screen.getByText("Paid into")).toBeInTheDocument();
    expect(screen.getByText("SBI Card", { selector: "li" })).toBeInTheDocument();
    expect(screen.getByText("ICICI Card", { selector: "li" })).toBeInTheDocument();
    expect(screen.getByText(/₹23,06,483.00 is still on its way/)).toBeInTheDocument();
    expect(screen.getByText("Matched as a card bill payment")).toBeInTheDocument();
  });

  it("says where the money came from on the Money in row, and what extra arrived", async () => {
    renderWithClient(<TransferBadge transfer={transfer({ other_accounts: [1], extra_paise: 500 })} direction="credit" names={names} />);
    await userEvent.click(screen.getByRole("button", { name: /Transfer → HDFC Savings/ }));
    expect(screen.getByText("Paid from")).toBeInTheDocument();
    expect(screen.getByText(/₹5.00 more arrived than was sent and counts as Money in/)).toBeInTheDocument();
  });

  it("unlinks the Transfer", async () => {
    const fetchMock = stubApi({ "DELETE /api/finance/transfers/9": () => new Response(null, { status: 204 }) });
    renderWithClient(<TransferBadge transfer={transfer()} direction="debit" names={names} />);
    await userEvent.click(screen.getByRole("button", { name: /Transfer → SBI Card/ }));

    await userEvent.click(screen.getByRole("button", { name: "Unlink" }));

    await vi.waitFor(() => { expect(requestsOf(fetchMock)).toContain("DELETE /api/finance/transfers/9"); });
  });

  it("explains why an unlink failed", async () => {
    stubApi({ "DELETE /api/finance/transfers/9": () => jsonResponse(404, { code: "transfer_not_found", detail: "There is no such Transfer." }) });
    renderWithClient(<TransferBadge transfer={transfer()} direction="debit" names={names} />);
    await userEvent.click(screen.getByRole("button", { name: /Transfer → SBI Card/ }));
    await userEvent.click(screen.getByRole("button", { name: "Unlink" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("There is no such Transfer.");
  });
});
