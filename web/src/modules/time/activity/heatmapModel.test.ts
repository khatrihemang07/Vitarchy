import { intensity, maxCell, orderRows, peakCell } from "./heatmapModel";

const grid = Array.from({ length: 7 }, (_, day) => Array.from({ length: 24 }, (_, hour) => (day === 2 && hour === 21 ? 900 : 0)));

describe("orderRows", () => {
  it("keeps Monday first for a Monday week", () => {
    expect(orderRows(grid, "monday").map((r) => r.short)).toEqual(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]);
  });

  it("moves Sunday to the top for a Sunday week, keeping each row's own hours", () => {
    const rows = orderRows(grid, "sunday");
    expect(rows.map((r) => r.short)).toEqual(["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]);
    expect(rows.find((r) => r.name === "Wednesday")?.hours[21]).toBe(900);
  });

  it("copes with a missing grid", () => {
    expect(orderRows([], "monday")[0]?.hours).toEqual([]);
  });
});

describe("intensity", () => {
  it("is 0 for no time, 1 for the busiest and in between on a square-root scale", () => {
    expect(intensity(0, 100)).toBe(0);
    expect(intensity(100, 100)).toBe(1);
    expect(intensity(25, 100)).toBeCloseTo(0.5);
  });

  it("is 0 when nothing was recorded at all", () => {
    expect(intensity(10, 0)).toBe(0);
  });
});

describe("peakCell and maxCell", () => {
  it("find the busiest hour", () => {
    const rows = orderRows(grid, "monday");
    expect(maxCell(rows)).toBe(900);
    const peak = peakCell(rows);
    expect([peak?.row.name, peak?.hour, peak?.seconds]).toEqual(["Wednesday", 21, 900]);
  });

  it("find nothing in an empty grid", () => {
    const rows = orderRows([], "monday");
    expect(peakCell(rows)).toBeNull();
    expect(maxCell(rows)).toBe(0);
  });
});
