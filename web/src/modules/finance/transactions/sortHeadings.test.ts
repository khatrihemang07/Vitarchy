import { sortableHeadings } from "./sortHeadings";

describe("sortableHeadings", () => {
  it("makes only Date and Amount sortable and shows the current order on its column", () => {
    const headings = sortableHeadings("amount_asc", vi.fn());
    const sorts = Object.fromEntries(headings.map((h) => [h.id, h.sort?.order]));
    expect(sorts).toMatchObject({ date: null, amount: "asc" });
    expect(headings.filter((h) => h.sort).map((h) => h.id)).toEqual(["date", "amount"]);
  });

  it("reports which column was clicked", () => {
    const onSort = vi.fn();
    sortableHeadings("date_desc", onSort).find((h) => h.id === "amount")?.sort?.onSort();
    expect(onSort).toHaveBeenCalledWith("amount");
  });
});
