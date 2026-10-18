import { describeAway, formatHour, formatMoment, formatPerHour, formatShare } from "./timelineFormat";

describe("formatHour", () => {
  const at = (h: number) => new Date(2026, 9, 16, h);
  it("reads the hour in the chosen clock", () => {
    expect(formatHour(at(15), "h24")).toBe("15");
    expect(formatHour(at(15), "h12")).toBe("3 PM");
    expect(formatHour(at(0), "h12")).toBe("12 AM");
    expect(formatHour(at(0), "h24")).toBe("00");
  });
});

describe("formatShare", () => {
  it("rounds to a whole percent, and says so when it is under one", () => {
    expect(formatShare(0.428)).toBe("43%");
    expect(formatShare(0.004)).toBe("<1%");
    expect(formatShare(0)).toBe("0%");
  });
});

describe("formatPerHour", () => {
  it("shows one decimal, or a dash with no Active time", () => {
    expect(formatPerHour(12.34)).toBe("12.3");
    expect(formatPerHour(null)).toBe("—");
  });
});

describe("describeAway", () => {
  it("names each state in the Glossary's words", () => {
    expect(describeAway("no")).toBe("Active");
    expect(describeAway("yes")).toBe("Away");
    expect(describeAway("unknown")).toBe("Away unknown");
  });
});

describe("formatMoment", () => {
  const iso = new Date(2026, 9, 16, 15, 5).toISOString();
  it("shows just the time for a Day and the date too for a longer Period", () => {
    expect(formatMoment(iso, "h24", false)).toBe("15:05");
    expect(formatMoment(iso, "h24", true)).toBe("Fri 16 Oct, 15:05");
  });
  it("is a dash when there is none", () => {
    expect(formatMoment(null, "h24", false)).toBe("—");
  });
});
