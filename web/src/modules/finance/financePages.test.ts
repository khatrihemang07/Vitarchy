import { FINANCE_COMMAND_PAGES, FINANCE_TABS, financePageFor } from "./financePages";

describe("financePageFor", () => {
  it("finds a page by path, ignoring a trailing slash", () => {
    expect(financePageFor("/finance/accounts/").title).toBe("Accounts");
  });
  it("falls back to the Import page for anything unknown", () => {
    expect(financePageFor("/finance/elsewhere").tab).toBe("Import");
  });
});

describe("page lists", () => {
  it("derive the tabs and the command bar entries from one table", () => {
    expect(FINANCE_TABS.map((t) => t.to)).toEqual(["/finance", "/finance/accounts", "/finance/transactions"]);
    expect(FINANCE_COMMAND_PAGES.map((p) => p.id)).toEqual(["import", "accounts", "transactions"]);
  });
});
