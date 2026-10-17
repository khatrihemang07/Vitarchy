import { formatSpanRange, formatAge, formatCount, formatDay, formatDuration, formatSpan } from "./format";

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

describe("formatSpan", () => {
  it("writes hours and minutes", () => {
    expect(formatSpan(3900)).toBe("1h 5m");
    expect(formatSpan(7200)).toBe("2h");
    expect(formatSpan(1500)).toBe("25m");
  });

  it("shows seconds only for under a minute", () => {
    expect(formatSpan(40)).toBe("40s");
    expect(formatSpan(0)).toBe("0m");
  });

  it("rounds to the nearest minute without spilling into 60m", () => {
    expect(formatSpan(3570)).toBe("1h");
    expect(formatSpan(89)).toBe("1m");
    expect(formatSpan(90)).toBe("2m");
  });

  it("drops the sign", () => {
    expect(formatSpan(-3900)).toBe("1h 5m");
  });
});

describe("formatSpanRange", () => {
  it("shows the smallest to the largest span", () => {
    expect(formatSpanRange(5 * 3600, 9 * 3600)).toBe("5h – 9h");
  });

  it("shows a single value when both ends read the same", () => {
    expect(formatSpanRange(2700, 2700)).toBe("45m");
    expect(formatSpanRange(2700, 2710)).toBe("45m");
  });
});
