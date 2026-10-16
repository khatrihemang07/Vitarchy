import { addDays, eachDate, pickDay } from "./dates";

describe("eachDate", () => {
  it("lists every date inclusive, across a month end", () => {
    expect(eachDate("2026-02-27", "2026-03-02")).toEqual(["2026-02-27", "2026-02-28", "2026-03-01", "2026-03-02"]);
  });

  it("is one date for a one-day range and empty when reversed", () => {
    expect(eachDate("2026-05-01", "2026-05-01")).toEqual(["2026-05-01"]);
    expect(eachDate("2026-05-02", "2026-05-01")).toEqual([]);
  });
});

describe("addDays", () => {
  it("steps forwards and backwards over a year end", () => {
    expect(addDays("2025-12-31", 1)).toBe("2026-01-01");
    expect(addDays("2026-01-01", -1)).toBe("2025-12-31");
  });
});

describe("pickDay", () => {
  const active = ["2026-05-02", "2026-05-04"];

  it("defaults to the last Day with Activity", () => {
    expect(pickDay(undefined, "2026-05-01", "2026-05-07", active)).toBe("2026-05-04");
  });

  it("keeps the person's choice while it is inside the Period, even without Activity", () => {
    expect(pickDay("2026-05-03", "2026-05-01", "2026-05-07", active)).toBe("2026-05-03");
  });

  it("drops a choice outside the Period", () => {
    expect(pickDay("2026-06-01", "2026-05-01", "2026-05-07", active)).toBe("2026-05-04");
  });

  it("has no Day when there is no Activity and no choice", () => {
    expect(pickDay(undefined, "2026-05-01", "2026-05-07", [])).toBeUndefined();
  });
});
