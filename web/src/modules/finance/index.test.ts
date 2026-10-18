import { financeModule } from "./index";

describe("financeModule", () => {
  it("is mounted at /finance under the finance id with the Finance accent", () => {
    expect(financeModule.id).toBe("finance");
    expect(financeModule.navItem.to).toBe("/finance");
    expect(financeModule.accent).toBe("var(--module-finance)");
  });

  it("lists Import, Accounts and Transactions as pages for the command bar", () => {
    expect(financeModule.pages?.map((page) => page.to)).toEqual(["/finance", "/finance/accounts", "/finance/transactions"]);
  });
});
