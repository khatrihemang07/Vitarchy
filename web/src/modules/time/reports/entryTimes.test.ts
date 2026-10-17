import type { ReportEntry } from "@/api/time";
import { chronologicalEntries, describeEntry, formatClock } from "./entryTimes";

const entry = (id: number, start: string, stop: string): ReportEntry => ({
  toggl_id: id, start, stop, duration_s: 60, description: "", tags: [],
});

describe("formatClock", () => {
  it("honours the 12-hour and 24-hour clocks", () => {
    const at = new Date("2026-10-17T01:05:00Z"); // 18:05 in Los Angeles
    expect(formatClock(at, "h12")).toBe("6:05 PM");
    expect(formatClock(at, "h24")).toBe("18:05");
  });
});

describe("describeEntry", () => {
  // Instants are explicit UTC; the tests run in America/Los_Angeles (UTC-7 in October, UTC-8 from 1 Nov).
  it("shows start and stop on the entry's first Day", () => {
    const e = entry(1, "2026-10-16T16:00:00Z", "2026-10-16T17:30:00Z");
    expect(describeEntry(e, "h24")).toEqual({ date: "Fri 16 Oct", start: "09:00", stop: "10:30", crossesMidnight: false });
  });

  it("reads the Day in the local zone, not the UTC date", () => {
    // 06:30Z on the 17th is 23:30 on Friday the 16th here.
    const e = entry(2, "2026-10-17T06:30:00Z", "2026-10-17T08:15:00Z");
    expect(describeEntry(e, "h24")).toEqual({ date: "Fri 16 Oct", start: "23:30", stop: "01:15", crossesMidnight: true });
  });

  it("does not flag an entry that stops exactly at midnight", () => {
    const e = entry(3, "2026-10-17T05:00:00Z", "2026-10-17T07:00:00Z");
    expect(describeEntry(e, "h24")).toMatchObject({ start: "22:00", stop: "00:00", crossesMidnight: false });
  });

  it("flags an entry that stops a minute past midnight", () => {
    const e = entry(4, "2026-10-17T05:00:00Z", "2026-10-17T07:01:00Z");
    expect(describeEntry(e, "h24").crossesMidnight).toBe(true);
  });

  it("does not flag a zero-length entry sitting on midnight", () => {
    const e = entry(5, "2026-10-17T07:00:00Z", "2026-10-17T07:00:00Z");
    expect(describeEntry(e, "h24").crossesMidnight).toBe(false);
  });

  it("does not flag an entry that starts at midnight and runs to the next one", () => {
    const e = entry(6, "2026-10-17T07:00:00Z", "2026-10-18T07:00:00Z");
    expect(describeEntry(e, "h24").crossesMidnight).toBe(false);
  });

  it("follows the 25-hour Day when the clocks go back", () => {
    // 1 Nov 2026 runs from 07:00Z to 08:00Z on the 2nd.
    const wholeDay = entry(7, "2026-11-01T07:00:00Z", "2026-11-02T08:00:00Z");
    expect(describeEntry(wholeDay, "h24").crossesMidnight).toBe(false);
    const longer = entry(8, "2026-11-01T07:00:00Z", "2026-11-02T08:01:00Z");
    expect(describeEntry(longer, "h24").crossesMidnight).toBe(true);
    // 24 hours after midnight is only 23:00 on that 25-hour Day.
    const short = entry(9, "2026-11-01T07:00:00Z", "2026-11-02T07:00:00Z");
    expect(describeEntry(short, "h24")).toMatchObject({ stop: "23:00", crossesMidnight: false });
  });
});

describe("chronologicalEntries", () => {
  it("lists every bottom-level entry once, earliest first", () => {
    const a = entry(1, "2026-10-16T09:00:00Z", "2026-10-16T10:00:00Z");
    const b = entry(2, "2026-10-16T07:00:00Z", "2026-10-16T08:00:00Z");
    const groups = [
      { children: [], entries: [a] },
      { children: [{ children: [], entries: [b, a] }], entries: [] },
    ];
    expect(chronologicalEntries(groups).map((e) => e.toggl_id)).toEqual([2, 1]);
  });
});
