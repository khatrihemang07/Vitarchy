import { TIME_TABS, timePageFor } from "./timePages";

describe("timePageFor", () => {
  it("titles each Time page", () => {
    expect(timePageFor("/time").title).toBe("Time");
    expect(timePageFor("/time/reports").title).toBe("Reports");
  });

  it("ignores a trailing slash and falls back to Sources for anything unknown", () => {
    expect(timePageFor("/time/reports/").title).toBe("Reports");
    expect(timePageFor("/time/other").title).toBe("Time");
  });
});

describe("TIME_TABS", () => {
  it("lists each Time page's path and tab label, in page order, from TIME_PAGES", () => {
    expect(TIME_TABS).toEqual([
      { to: "/time", label: "Sources" },
      { to: "/time/reports", label: "Reports" },
    ]);
  });
});
