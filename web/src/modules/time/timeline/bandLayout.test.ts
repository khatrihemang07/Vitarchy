import { dayBounds, hourTicks, layoutBand, MIN_WIDTH_PCT, nowPct, SLIVER_PCT } from "./bandLayout";

const bounds = { startMs: Date.parse("2026-10-16T00:00:00Z"), endMs: Date.parse("2026-10-17T00:00:00Z") };
const seg = (from: string, to: string, away = "no") => ({ start_at: `2026-10-16T${from}:00Z`, end_at: `2026-10-16T${to}:00Z`, away }) as never;

describe("layoutBand", () => {
  it("places a Segment by its share of the Day", () => {
    const [mark] = layoutBand([seg("06:00", "12:00")], bounds);
    expect(mark).toMatchObject({ left: 25, width: 25, segments: [0] });
  });

  it("gives a short Segment a minimum visible width", () => {
    const [mark] = layoutBand([seg("06:00", "06:05")], bounds);
    expect(mark?.width).toBeGreaterThanOrEqual(MIN_WIDTH_PCT);
  });

  it("merges consecutive slivers into one mark for drawing only, keeping every Segment reachable", () => {
    const sliver = (m: number) => seg(`06:${String(m).padStart(2, "0")}`, `06:${String(m + 1).padStart(2, "0")}`);
    expect((1 / 1440) * 100).toBeLessThan(SLIVER_PCT);
    const marks = layoutBand([sliver(0), sliver(1), sliver(2)], bounds);
    expect(marks).toHaveLength(1);
    expect(marks[0]?.segments).toEqual([0, 1, 2]);
  });

  it("does not merge slivers that have a gap between them, so no colour covers time without data", () => {
    const marks = layoutBand([seg("06:00", "06:01"), seg("06:05", "06:06")], bounds);
    expect(marks.map((b) => b.segments)).toEqual([[0], [1]]);
  });

  it("does not merge touching slivers whose Away differs", () => {
    const marks = layoutBand([seg("06:00", "06:01", "no"), seg("06:01", "06:02", "yes"), seg("06:02", "06:03", "yes")], bounds);
    expect(marks.map((b) => b.segments)).toEqual([[0], [1, 2]]);
  });

  it("does not merge a sliver into a long Segment", () => {
    const marks = layoutBand([seg("06:00", "06:01"), seg("06:01", "09:00")], bounds);
    expect(marks.map((b) => b.segments)).toEqual([[0], [1]]);
  });

  it("clamps a Segment that runs past the Day and skips one outside it", () => {
    const marks = layoutBand([{ start_at: "2026-10-15T23:00:00Z", end_at: "2026-10-16T01:00:00Z", away: "no" } as never, { start_at: "2026-10-18T01:00:00Z", end_at: "2026-10-18T02:00:00Z", away: "no" } as never], bounds);
    expect(marks).toHaveLength(1);
    expect(marks[0]).toMatchObject({ left: 0 });
    expect(marks[0]?.width).toBeCloseTo(100 / 24);
  });
});

describe("dayBounds", () => {
  it("runs from local midnight to the next local midnight", () => {
    const { startMs, endMs } = dayBounds("2026-10-16");
    expect(new Date(startMs).getHours()).toBe(0);
    expect(new Date(endMs).getDate()).toBe(17);
    expect(endMs - startMs).toBeGreaterThanOrEqual(23 * 3_600_000);
  });
});

describe("hourTicks", () => {
  it("marks every third hour from midnight to the next midnight", () => {
    const ticks = hourTicks("2026-10-16", 3);
    expect(ticks).toHaveLength(9);
    expect(ticks[0]?.pct).toBe(0);
    expect(ticks[8]?.pct).toBe(100);
    expect(ticks[4]?.date.getHours()).toBe(12);
  });
});

describe("hourTicks on a Day the clocks change", () => {
  const original = process.env.TZ;
  afterEach(() => {
    if (original === undefined) delete process.env.TZ;
    else process.env.TZ = original;
  });

  it("never repeats a time on the spring-forward Day", () => {
    process.env.TZ = "America/New_York";
    const ticks = hourTicks("2026-03-08", 1);
    const times = ticks.map((t) => t.date.getTime());
    expect(new Set(times).size).toBe(times.length);
    expect(ticks.at(-1)?.pct).toBe(100);
  });
});

describe("nowPct", () => {
  it("is the position of now inside the Day and null outside it", () => {
    expect(nowPct(bounds, Date.parse("2026-10-16T12:00:00Z"))).toBe(50);
    expect(nowPct(bounds, Date.parse("2026-10-17T12:00:00Z"))).toBeNull();
  });
});
