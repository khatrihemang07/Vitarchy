import type { TransferInfo } from "@/api/finance";
import { transferBadgeLabel, transferRungLabel, transferStatusLine } from "./transferCopy";

const names = new Map([[1, "HDFC Savings"], [2, "SBI Card"], [3, "ICICI Card"], [4, "Loan"]]);
const info = (over: Partial<TransferInfo> = {}): TransferInfo => ({
  id: 1, rung: "bill_payment", other_accounts: [2], partial: false, remaining_paise: 0, extra_paise: 0, ...over,
});

describe("transferBadgeLabel", () => {
  it("names the one Account on the other side", () => {
    expect(transferBadgeLabel(info(), names)).toBe("Transfer → SBI Card");
  });

  it("names two Accounts and counts the rest", () => {
    expect(transferBadgeLabel(info({ other_accounts: [2, 3] }), names)).toBe("Transfer → SBI Card, ICICI Card");
    expect(transferBadgeLabel(info({ other_accounts: [2, 3, 4] }), names)).toBe("Transfer → SBI Card, ICICI Card +1");
  });

  it("falls back to a plain word for an Account it does not know", () => {
    expect(transferBadgeLabel(info({ other_accounts: [99] }), names)).toBe("Transfer → Account");
  });
});

describe("transferStatusLine", () => {
  it("says nothing for a complete Transfer", () => {
    expect(transferStatusLine(info())).toBeNull();
  });

  it("says what is still on its way for a partial one", () => {
    expect(transferStatusLine(info({ partial: true, remaining_paise: 230_648_300 }))).toBe(
      "₹23,06,483.00 is still on its way and counts as Money out until it arrives.",
    );
  });

  it("says the extra counts as Money in", () => {
    expect(transferStatusLine(info({ extra_paise: 450 }))).toBe("₹4.50 more arrived than was sent and counts as Money in.");
  });
});

describe("transferRungLabel", () => {
  it("explains how each Transfer was linked, in words", () => {
    expect(transferRungLabel("reference")).toBe("Matched by its reference");
    expect(transferRungLabel("account_number")).toBe("Matched by the Account number in its description");
    expect(transferRungLabel("bill_payment")).toBe("Matched as a card bill payment");
    expect(transferRungLabel("unique_amount")).toBe("Matched by its amount, the only one that day");
    expect(transferRungLabel("manual")).toBe("Linked by you");
  });
});
