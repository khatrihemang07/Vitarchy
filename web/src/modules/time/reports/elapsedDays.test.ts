import { elapsedDays } from "./elapsedDays";

const week = { from: "2026-10-12", to: "2026-10-18" };

describe("elapsedDays", () => {
  it("counts a past Period in full", () => {
    expect(elapsedDays(week, "2026-10-25")).toBe(7);
  });

  it("stops at today inside the current Period", () => {
    expect(elapsedDays(week, "2026-10-14")).toBe(3);
  });

  it("counts the first Day of a Period that has just started as 1", () => {
    expect(elapsedDays(week, "2026-10-12")).toBe(1);
  });

  it("counts the last Day as the full Period", () => {
    expect(elapsedDays(week, "2026-10-18")).toBe(7);
  });

  it("is 0 for a Period that has not started", () => {
    expect(elapsedDays(week, "2026-10-11")).toBe(0);
  });
});
