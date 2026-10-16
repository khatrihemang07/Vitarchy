import { formatAge, formatCount, formatDay, formatDuration } from "./format";

const now = new Date("2026-04-10T12:00:00Z");
const ago = (secs: number) => new Date(now.getTime() - secs * 1000);

describe("formatAge", () => {
  it("calls anything under a minute 'just now'", () => {
    expect(formatAge(ago(5), now)).toBe("just now");
  });

  it("counts minutes, hours and days", () => {
    expect(formatAge(ago(5 * 60), now)).toBe("5 min ago");
    expect(formatAge(ago(3 * 3600), now)).toBe("3 h ago");
    expect(formatAge(ago(2 * 86_400), now)).toBe("2 d ago");
  });

  it("accepts an ISO string and treats the future as 'just now'", () => {
    expect(formatAge("2026-04-10T11:00:00Z", now)).toBe("1 h ago");
    expect(formatAge("2026-04-10T13:00:00Z", now)).toBe("just now");
  });
});

describe("formatDuration", () => {
  it("rounds up to the unit", () => {
    expect(formatDuration(0.2)).toBe("1 s");
    expect(formatDuration(40)).toBe("40 s");
    expect(formatDuration(61)).toBe("2 min");
    expect(formatDuration(1800)).toBe("30 min");
    expect(formatDuration(5400)).toBe("2 h");
  });
});

describe("formatDay", () => {
  it("shows the date in the person's locale", () => {
    expect(formatDay("2026-04-10T12:00:00Z")).toMatch(/2026/);
  });
});

describe("formatCount", () => {
  it("groups thousands so large counts stay readable", () => {
    expect(formatCount(10145)).toBe(new Intl.NumberFormat().format(10145));
    expect(formatCount(10145)).not.toBe("10145");
  });

  it("leaves small counts as they are", () => {
    expect(formatCount(7)).toBe("7");
  });
});
