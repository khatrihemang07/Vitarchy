import { rowContextFor } from "./rowContext";
import { report } from "./testReport";

const base = { mode: "per_day", compare: "previous", clock: "h24" } as const;

describe("rowContextFor", () => {
  it("counts only the Days of the Period that have happened, as the server says", () => {
    expect(rowContextFor({ ...base, period: { ...report([]).period, elapsed_days: 3 } }).periodDays).toBe(3);
  });

  it("marks a one-Day Report and picks its columns from the comparison", () => {
    const day = report([], "day").period;
    expect(rowContextFor({ ...base, period: day }).isDay).toBe(true);
    expect(rowContextFor({ ...base, period: day }).layout.columns.map((c) => c.label)).toEqual(["Entries", "Total"]);
    expect(rowContextFor({ ...base, period: day, compare: "usual" }).layout.columns.map((c) => c.label)).toContain("Usual range");
  });
});
