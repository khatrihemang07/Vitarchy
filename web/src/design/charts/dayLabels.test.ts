import { axisLabel, fullDayLabel } from "./dayLabels";

describe("fullDayLabel", () => {
  it("names the weekday, day and month", () => {
    expect(fullDayLabel("2026-10-16")).toBe("Fri 16 Oct");
  });
});

describe("axisLabel", () => {
  it("uses weekday names when there are few bars", () => {
    expect(axisLabel("2026-10-16", 7)).toBe("Fri");
  });

  it("uses the day of the month when there are many bars", () => {
    expect(axisLabel("2026-10-16", 31)).toBe("16");
  });
});
