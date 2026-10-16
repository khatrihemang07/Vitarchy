import { formatHourOfDay, formatInstant, formatMinuteOfDay } from "./timeOfDay";

describe("formatMinuteOfDay", () => {
  it("writes 12-hour and 24-hour clocks as asked", () => {
    expect(formatMinuteOfDay(9 * 60 + 5, "h12", "en-US")).toMatch(/^9:05\s?AM$/);
    expect(formatMinuteOfDay(21 * 60 + 30, "h24", "en-US")).toBe("21:30");
    expect(formatMinuteOfDay(9 * 60 + 5, "h24", "en-US")).toBe("09:05");
  });

  it("follows the locale when the clock is the system's", () => {
    expect(formatMinuteOfDay(15 * 60, "system", "en-US")).toMatch(/^3:00\s?PM$/);
    expect(formatMinuteOfDay(15 * 60, "system", "en-GB")).toBe("15:00");
  });

  it("writes the end of the day as 24:00 on every clock, so it is never mistaken for the start", () => {
    expect(formatMinuteOfDay(24 * 60, "h24", "en-US")).toBe("24:00");
    expect(formatMinuteOfDay(24 * 60, "h12", "en-US")).toBe("24:00");
    expect(formatMinuteOfDay(24 * 60, "system", "en-US")).toBe("24:00");
  });
});

describe("formatHourOfDay", () => {
  it("writes a whole hour", () => {
    expect(formatHourOfDay(14, "h12", "en-US")).toMatch(/^2\s?PM$/);
    expect(formatHourOfDay(14, "h24", "en-US")).toBe("14");
  });

  it("writes hour 24 as 24:00 on every clock", () => {
    expect(formatHourOfDay(24, "h12", "en-US")).toBe("24:00");
    expect(formatHourOfDay(24, "h24", "en-US")).toBe("24:00");
  });
});

describe("formatInstant", () => {
  it("shows the time in the given zone", () => {
    expect(formatInstant("2026-05-01T04:00:00Z", "Asia/Kolkata", "h24", "en-US")).toBe("09:30");
  });
});
