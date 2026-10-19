import { orderOf, readSort, toggleSort } from "./sortColumns";

describe("readSort", () => {
  it("splits a sort into its column and direction", () => {
    expect(readSort("date_desc")).toEqual({ column: "date", order: "desc" });
    expect(readSort("amount_asc")).toEqual({ column: "amount", order: "asc" });
  });
});

describe("toggleSort", () => {
  it("starts a new column newest or largest first", () => {
    expect(toggleSort("date_desc", "amount")).toBe("amount_desc");
    expect(toggleSort("amount_asc", "date")).toBe("date_desc");
  });
  it("flips the direction of the column already sorted", () => {
    expect(toggleSort("date_desc", "date")).toBe("date_asc");
    expect(toggleSort("date_asc", "date")).toBe("date_desc");
    expect(toggleSort("amount_desc", "amount")).toBe("amount_asc");
  });
});

describe("orderOf", () => {
  it("shows a direction only for the sorted column", () => {
    expect(orderOf("amount_asc", "amount")).toBe("asc");
    expect(orderOf("amount_asc", "date")).toBeNull();
  });
});
