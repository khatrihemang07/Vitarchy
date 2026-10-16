import { colorLabels, placeSegments, segmentSeconds, type TimelineSegment } from "./timelineLayout";

const DAY = ["2026-05-01T00:00:00Z", "2026-05-02T00:00:00Z"] as const;
const segment = (start: string, end: string, label = "A"): TimelineSegment => ({ start, end, label });

describe("placeSegments", () => {
  it("places a segment by its share of the Day", () => {
    const [placed] = placeSegments([segment("2026-05-01T06:00:00Z", "2026-05-01T12:00:00Z")], ...DAY);
    expect(placed?.left).toBeCloseTo(25);
    expect(placed?.width).toBeCloseTo(25);
  });

  it("cuts off what lies outside the Day and drops segments wholly outside", () => {
    const placed = placeSegments(
      [segment("2026-04-30T20:00:00Z", "2026-05-01T06:00:00Z"), segment("2026-05-03T00:00:00Z", "2026-05-03T01:00:00Z")],
      ...DAY,
    );
    expect(placed).toHaveLength(1);
    expect(placed[0]?.left).toBe(0);
    expect(placed[0]?.width).toBeCloseTo(25);
  });

  it("places nothing on a Day of no length", () => {
    expect(placeSegments([segment(DAY[0], DAY[1])], DAY[0], DAY[0])).toEqual([]);
  });

  it("measures a segment in seconds", () => {
    expect(segmentSeconds(segment("2026-05-01T06:00:00Z", "2026-05-01T06:01:30Z"))).toBe(90);
  });
});

describe("colorLabels", () => {
  it("gives the longest label the first colour and sums repeats", () => {
    const entries = colorLabels([
      segment("2026-05-01T01:00:00Z", "2026-05-01T01:10:00Z", "Short"),
      segment("2026-05-01T02:00:00Z", "2026-05-01T03:00:00Z", "Long"),
      segment("2026-05-01T04:00:00Z", "2026-05-01T04:10:00Z", "Short"),
    ]);
    expect(entries.map((e) => [e.label, e.seconds])).toEqual([["Long", 3600], ["Short", 1200]]);
    expect(entries[0]?.color).toBe("var(--chart-1)");
    expect(entries[1]?.color).toBe("var(--chart-2)");
  });

  it("shares the neutral colour among labels past the palette", () => {
    const many = Array.from({ length: 10 }, (_, i) =>
      segment(`2026-05-01T0${String(i)}:00:00Z`, `2026-05-01T0${String(i)}:${String(50 - i * 4)}:00Z`, `L${String(i)}`),
    );
    const colors = colorLabels(many).map((e) => e.color);
    expect(new Set(colors.slice(0, 8)).size).toBe(8);
    expect(colors[8]).toBe("var(--chart-other)");
    expect(colors[9]).toBe("var(--chart-other)");
  });
});
