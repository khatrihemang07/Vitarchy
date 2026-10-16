import type { Insight } from "@/api/time";
import { describeInsight } from "./insightText";

function insight(parts: Partial<Insight> & Pick<Insight, "kind">, numbers: Partial<Insight["numbers"]> = {}): Insight {
  return {
    subject: { kind: "routine_step", name: "Office" },
    measure: "length",
    score: 50,
    days: [],
    ...parts,
    numbers: { value: 0, samples: 10, ...numbers },
  };
}

const text = (i: Insight) => describeInsight(i, "h24");

describe("describeInsight", () => {
  it("says when a Step usually starts or finishes, with the range most Days fall in", () => {
    const numbers = { value: 9 * 60 + 36, p25: 9 * 60 + 31, p75: 9 * 60 + 41 };
    expect(text(insight({ kind: "typical_time", measure: "start" }, numbers))).toBe("You usually start Office at 09:36 (09:31 – 09:41)");
    expect(text(insight({ kind: "typical_time", measure: "end" }, numbers))).toBe("You usually finish Office at 09:36 (09:31 – 09:41)");
  });

  it("follows the clock setting", () => {
    const numbers = { value: 17 * 60 + 5, p25: 17 * 60, p75: 17 * 60 + 10 };
    expect(describeInsight(insight({ kind: "typical_time", measure: "end" }, numbers), "h12")).toContain("5:05 PM (5:00 PM – 5:10 PM)");
  });

  it("describes a usual length for each kind of subject", () => {
    const numbers = { value: 32, p25: 28, p75: 38 };
    expect(text(insight({ kind: "typical_duration" }, numbers))).toBe("Office usually takes 32 min (28 min – 38 min)");
    expect(text(insight({ kind: "typical_duration", subject: { kind: "routine_total", name: "Office day" } }, { value: 525, p25: 500, p75: 540 }))).toBe(
      "Office day usually takes 8 h 45 min door to door (8 h 20 min – 9 h)",
    );
    expect(text(insight({ kind: "typical_duration", subject: { kind: "app", name: "Brave" } }, { value: 180, p25: 150, p75: 200 }))).toBe(
      "You usually spend 3 h a day in Brave (2 h 30 min – 3 h 20 min)",
    );
  });

  it("compares a weekday's length with the usual", () => {
    expect(text(insight({ kind: "weekday_difference" }, { value: 430, reference: 480, delta: -50, weekday: 4 }))).toBe("On Fridays, Office takes 50 min less than usual");
    expect(text(insight({ kind: "weekday_difference", subject: { kind: "app", name: "Brave" } }, { value: 150, reference: 120, delta: 30, weekday: 0 }))).toBe(
      "On Mondays, you spend 30 min more a day in Brave than usual",
    );
  });

  it("compares a weekday's time of day with the usual", () => {
    expect(text(insight({ kind: "weekday_difference", measure: "end" }, { delta: -50, weekday: 4 }))).toBe("On Fridays you finish Office 50 min earlier than usual");
    expect(text(insight({ kind: "weekday_difference", measure: "start" }, { delta: 40, weekday: 6 }))).toBe("On Sundays you start Office 40 min later than usual");
  });

  it("compares the last four weeks with the four before", () => {
    expect(text(insight({ kind: "change" }, { value: 42, reference: 30, delta: 12 }))).toBe("Office took 12 min longer than in the previous 4 weeks");
    expect(text(insight({ kind: "change", subject: { kind: "app", name: "Brave" } }, { delta: -60 }))).toBe("You spent 1 h less a day in Brave than in the previous 4 weeks");
  });

  it("names the Day of the longest or shortest length", () => {
    const base = { kind: "outlier" as const, days: ["2026-05-12"] };
    expect(text(insight(base, { value: 612, reference: 480, delta: 132 }))).toBe("Longest Office day: Tue, May 12, 10 h 12 min");
    expect(text(insight(base, { value: 300, reference: 480, delta: -180 }))).toBe("Shortest Office day: Tue, May 12, 5 h");
    expect(text(insight({ ...base, subject: { kind: "context", name: "YouTube" } }, { value: 300, delta: 120 }))).toBe("Most time in YouTube: Tue, May 12, 5 h");
  });

  it("names the Day of the latest or earliest time", () => {
    const base = { kind: "outlier" as const, days: ["2026-05-12"] };
    expect(text(insight({ ...base, measure: "start" }, { value: 600, reference: 540, delta: 60 }))).toBe("Latest start of Office: Tue, May 12, 10:00 (usually 09:00)");
    expect(text(insight({ ...base, measure: "end" }, { value: 960, reference: 1020, delta: -60 }))).toBe("Earliest finish of Office: Tue, May 12, 16:00 (usually 17:00)");
  });
});
