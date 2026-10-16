import { describeHabit } from "./habitText";

const stat = {
  description: "inside_office",
  days: 21,
  entries: 21,
  start: { median_minute: 9 * 60 + 10, p25_minute: 8 * 60 + 55, p75_minute: 9 * 60 + 30 },
  end: { median_minute: 17 * 60 + 30, p25_minute: 17 * 60, p75_minute: 18 * 60 },
  duration: { median_seconds: 32 * 60, p25_seconds: 28 * 60, p75_seconds: 38 * 60 },
};

describe("describeHabit", () => {
  it("writes the usual start, end and length with their ranges on the chosen clock", () => {
    const text = describeHabit(stat, "h24");
    expect(text.start).toEqual({ usual: "09:10", range: "08:55 – 09:30" });
    expect(text.end).toEqual({ usual: "17:30", range: "17:00 – 18:00" });
    expect(text.duration).toEqual({ usual: "32 min", range: "28 min – 38 min" });
    expect(text.days).toBe("21 days");
  });

  it("keeps a range that crosses midnight in clock order", () => {
    const night = { ...stat, start: { median_minute: 0, p25_minute: 23 * 60 + 40, p75_minute: 25 } };
    expect(describeHabit(night, "h24").start).toEqual({ usual: "00:00", range: "23:40 – 00:25" });
  });

  it("says 1 day and notes when an entry is not the only one a day", () => {
    expect(describeHabit({ ...stat, days: 1, entries: 1 }, "h24").days).toBe("1 day");
    expect(describeHabit({ ...stat, days: 5, entries: 8 }, "h24").days).toBe("5 days (8 times)");
  });
});
