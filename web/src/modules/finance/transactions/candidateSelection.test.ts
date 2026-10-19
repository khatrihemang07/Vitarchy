import type { TransactionItem, TransferCandidate } from "@/api/finance";
import { linkRequest, selectedPaise, toggleCandidate } from "./candidateSelection";

const candidate = (id: number, amount: number, direction: "debit" | "credit"): TransferCandidate => ({
  id, account_id: 2, date: "2026-07-05", description: "x", amount_paise: amount, direction,
});

describe("toggleCandidate", () => {
  it("adds and removes when several may be chosen", () => {
    expect(toggleCandidate([1], 2, true)).toEqual([1, 2]);
    expect(toggleCandidate([1, 2], 1, true)).toEqual([2]);
  });

  it("keeps only the latest choice when just one may be chosen", () => {
    expect(toggleCandidate([1], 2, false)).toEqual([2]);
    expect(toggleCandidate([2], 2, false)).toEqual([]);
  });
});

describe("selectedPaise", () => {
  it("adds up what is chosen", () => {
    const all = [candidate(1, 400, "credit"), candidate(2, 600, "credit"), candidate(3, 50, "credit")];
    expect(selectedPaise(all, [1, 2])).toBe(1_000);
    expect(selectedPaise(all, [])).toBe(0);
  });
});

describe("linkRequest", () => {
  const row = (id: number, direction: "debit" | "credit") => ({ id, direction }) as TransactionItem;

  it("puts a Money-out row first and the chosen Money-in rows as its credits", () => {
    expect(linkRequest(row(10, "debit"), [20, 21])).toEqual({ debit_id: 10, credit_ids: [20, 21] });
  });

  it("makes the chosen Money-out Transaction the debit of a Money-in row", () => {
    expect(linkRequest(row(20, "credit"), [10])).toEqual({ debit_id: 10, credit_ids: [20] });
  });
});
