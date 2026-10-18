import type { AccountSummary, SectionPreview } from "@/api/finance";
import { initialDraft, isDraftComplete, pickerAccounts, suggestedName, toSectionChoice } from "./sectionChoice";

function section(over: Partial<SectionPreview> = {}): SectionPreview {
  return {
    position: 0,
    proposed: { issuer: "HDFC", kind: "credit_card", mask: "XXXX XXXX 1234" },
    matched_account: null,
    matched_without_number: false,
    candidate_accounts: [],
    period: {},
    transaction_count: 0,
    total_debit_paise: 0,
    total_credit_paise: 0,
    checks: [],
    new_count: 0,
    duplicate_count: 0,
    transactions: [],
    ...over,
  };
}

describe("initialDraft", () => {
  it("starts on the matched Account", () => {
    const draft = initialDraft(section({ matched_account: { id: 7, name: "Card" } }));
    expect(draft).toEqual({ mode: "existing", accountId: 7 });
  });
  it("asks the person to choose when several Accounts fit", () => {
    const draft = initialDraft(section({ candidate_accounts: [{ id: 1, name: "A" }, { id: 2, name: "B" }] }));
    expect(draft).toEqual({ mode: "existing", accountId: null });
    expect(isDraftComplete(draft)).toBe(false);
  });
  it("fills a New Account from what the reader found", () => {
    const draft = initialDraft(section());
    expect(draft).toEqual({ mode: "new", account: { issuer: "HDFC", kind: "credit_card", name: "HDFC Credit card 1234" } });
    expect(isDraftComplete(draft)).toBe(true);
  });
  it("leaves the kind empty when the reader found none, so it must be chosen", () => {
    const draft = initialDraft(section({ proposed: { issuer: null, kind: null, mask: null } }));
    expect(isDraftComplete(draft)).toBe(false);
  });
});

describe("toSectionChoice", () => {
  it("sends an existing Account by id", () => {
    expect(toSectionChoice({ mode: "existing", accountId: 3 })).toEqual({ account_id: 3 });
  });
  it("sends a New Account trimmed", () => {
    expect(toSectionChoice({ mode: "new", account: { issuer: " SBI ", kind: "loan", name: " Home " } })).toEqual({
      new_account: { issuer: "SBI", kind: "loan", name: "Home" },
    });
  });
});

describe("suggestedName", () => {
  it("uses only the last four digits of the number", () => {
    expect(suggestedName(section({ proposed: { issuer: "SBI", kind: "savings", mask: "00000012345678" } }))).toBe(
      "SBI Savings account 5678",
    );
  });
});

describe("pickerAccounts", () => {
  it("lists the Accounts that fit before every other Account, once each", () => {
    const all = [{ id: 1, name: "A" }, { id: 2, name: "B" }, { id: 3, name: "C" }] as AccountSummary[];
    const picked = pickerAccounts(section({ matched_account: { id: 2, name: "B" } }), all);
    expect(picked.map((a) => a.id)).toEqual([2, 1, 3]);
  });
});
