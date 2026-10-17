import type { ReportEntry } from "@/api/time";
import { chronologicalEntries, describeEntry, formatClock } from "./entryTimes";

const entry = (id: number, start: string, stop: string): ReportEntry => ({
  toggl_id: id, start, stop, duration_s: 60, description: "", tags: [],
});

describe("formatClock", () => {
  it("honours the 12-hour and 24-hour clocks", () => {
    const at = new Date(2026, 9, 16, 18, 5);
    expect(formatClock(at, "h12")).toBe("6:05 PM");
    expect(formatClock(at, "h24")).toBe("18:05");
  });
});

describe("describeEntry", () => {
  it("shows start and stop on the entry's first Day", () => {
    const e = entry(1, new Date(2026, 9, 16, 9, 0).toISOString(), new Date(2026, 9, 16, 10, 30).toISOString());
    expect(describeEntry(e, "h24")).toEqual({ date: "Fri 16 Oct", start: "09:00", stop: "10:30", crossesMidnight: false });
  });

  it("marks an entry that stops on a later Day", () => {
    const e = entry(2, new Date(2026, 9, 16, 23, 30).toISOString(), new Date(2026, 9, 17, 1, 15).toISOString());
    expect(describeEntry(e, "h24")).toMatchObject({ start: "23:30", stop: "01:15", crossesMidnight: true });
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
