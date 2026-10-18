import { account } from "../test-utils";
import { accountFigures } from "./accountFigures";

describe("accountFigures", () => {
  it("derives the balance, counts and date coverage", () => {
    const figures = Object.fromEntries(accountFigures(account({ transaction_count: 1234, statement_count: 1 })).map((f) => [f.label, f.value]));
    expect(figures["Latest balance"]).toBe("₹98,765.00");
    expect(figures.Transactions).toBe("1,234 Transactions");
    expect(figures.Coverage).toBe("1 Jan 2026 to 30 Apr 2026");
    expect(figures.Statements).toBe("1 Statement");
  });

  it("says so when no balance was printed and nothing is stored", () => {
    const figures = Object.fromEntries(
      accountFigures(account({ latest_balance_paise: null, first_date: null, last_date: null, transaction_count: 0 })).map((f) => [f.label, f.value]),
    );
    expect(figures["Latest balance"]).toBe("Not printed");
    expect(figures.Coverage).toBe("No Transactions yet");
  });
});
