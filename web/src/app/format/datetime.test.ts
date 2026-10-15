import { dateFormatLabel, formatDate, formatTime } from "./datetime";

const evening = new Date(2026, 9, 16, 18, 5);
const morning = new Date(2026, 2, 3, 9, 7);

describe("formatDate", () => {
  it.each([
    ["system", "Friday, October 16, 2026"],
    ["day_month_year", "Friday, 16 October 2026"],
    ["month_day_year", "Friday, October 16, 2026"],
    ["iso", "2026-10-16"],
  ] as const)("%s writes %s", (date_format, expected) => {
    expect(formatDate(evening, { date_format }, "en-US")).toBe(expected);
  });

  it("zero-pads the ISO form", () => {
    expect(formatDate(morning, { date_format: "iso" }, "en-US")).toBe("2026-03-03");
  });

  it("lets the locale shape the system format", () => {
    expect(formatDate(evening, { date_format: "system" }, "de-DE")).toContain("Oktober");
  });
});

describe("formatTime", () => {
  it.each([
    ["h12", "6:05 PM"],
    ["h24", "18:05"],
    ["system", "6:05 PM"],
  ] as const)("%s writes %s", (clock, expected) => {
    expect(formatTime(evening, { clock }, "en-US")).toBe(expected);
  });

  it("does not pad the 12-hour form but pads the 24-hour one", () => {
    expect(formatTime(morning, { clock: "h12" }, "en-US")).toBe("9:07 AM");
    expect(formatTime(morning, { clock: "h24" }, "en-US")).toBe("09:07");
  });
});

describe("dateFormatLabel", () => {
  it.each([
    ["system", "System (Friday, October 16, 2026)"],
    ["day_month_year", "Friday, 16 October 2026"],
    ["month_day_year", "Friday, October 16, 2026"],
    ["iso", "2026-10-16"],
  ] as const)("%s is labelled %s", (format, expected) => {
    expect(dateFormatLabel(format, "en-US")).toBe(expected);
  });

  it("follows the locale for the system format only", () => {
    expect(dateFormatLabel("system", "de-DE")).toContain("Oktober");
    expect(dateFormatLabel("iso", "de-DE")).toBe("2026-10-16");
  });
});
