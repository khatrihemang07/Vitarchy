import { formatBalance, formatCount, formatInr, formatSignedAmount, isCredit } from "./money";

describe("formatInr", () => {
  it.each([
    [0, "₹0.00"],
    [5, "₹0.05"],
    [100, "₹1.00"],
    [99_999, "₹999.99"],
    [100_000, "₹1,000.00"],
    [12_345_678, "₹1,23,456.78"],
    [123_456_789_00, "₹12,34,56,789.00"],
    [-250_050, "−₹2,500.50"],
  ])("writes %i paise as %s", (paise, expected) => {
    expect(formatInr(paise)).toBe(expected);
  });
});

describe("formatSignedAmount", () => {
  it("marks a debit with a minus and a credit with a plus", () => {
    expect(formatSignedAmount(50_000, "debit")).toBe("−₹500.00");
    expect(formatSignedAmount(50_000, "credit")).toBe("+₹500.00");
  });
});

describe("formatCount", () => {
  it("groups in the Indian way", () => {
    expect(formatCount(1_234_567)).toBe("12,34,567");
  });
});

describe("isCredit", () => {
  it("is true only for money coming in", () => {
    expect(isCredit("credit")).toBe(true);
    expect(isCredit("debit")).toBe(false);
  });
});

describe("formatBalance", () => {
  it("writes a printed balance in INR and nothing for a missing one", () => {
    expect(formatBalance(9_876_500)).toBe("₹98,765.00");
    expect(formatBalance(null)).toBe("");
    expect(formatBalance(undefined)).toBe("");
  });
});
