import { hoursAxis, splitChart } from "./splitModel";

const colors = new Map([["office", "#2a78d6"], ["sleep", "var(--chart-1)"]]);
const split = [
  { date: "2026-05-04", elapsed_seconds: 86400, unlogged_seconds: 43200, by_project: [{ project: "office", seconds: 28800 }, { project: "sleep", seconds: 14400 }] },
  { date: "2026-05-05", elapsed_seconds: 0, unlogged_seconds: 0, by_project: [] },
];
const projects = [{ name: "office" }, { name: "sleep" }];

describe("splitChart", () => {
  it("makes one stacked series per project in the given order, then the unlogged remainder", () => {
    const { series } = splitChart(split, projects, colors);
    expect(series.map((s) => [s.label, s.color])).toEqual([["office", "#2a78d6"], ["sleep", "var(--chart-1)"], ["Unlogged", "var(--chart-unlogged)"]]);
  });

  it("gives each Day's hours for the chart and its exact seconds for the words", () => {
    const { rows, series } = splitChart(split, projects, colors);
    const [office, sleep, unlogged] = series;
    expect(rows[0]).toMatchObject({ date: "2026-05-04", [office?.key ?? ""]: 8, [sleep?.key ?? ""]: 4, [unlogged?.key ?? ""]: 12 });
    expect(rows[0]?.seconds[office?.key ?? ""]).toBe(28800);
  });

  it("leaves a project that took no time on a Day at zero, and a Day yet to come empty", () => {
    const { rows, series } = splitChart(split, projects, colors);
    expect(rows[1]?.[series[0]?.key ?? ""]).toBe(0);
    expect(rows[1]?.[series[2]?.key ?? ""]).toBe(0);
  });
});

describe("hoursAxis", () => {
  it("runs 0 to 24 in steps of six for ordinary Days", () => {
    const { rows } = splitChart(split, projects, colors);
    expect(hoursAxis(rows)).toEqual({ top: 24, ticks: [0, 6, 12, 18, 24] });
  });

  it("grows in whole steps when overlapping entries pass 24 hours", () => {
    const rows = [{ date: "d", seconds: { a: 0 }, a: 13, b: 14 }];
    expect(hoursAxis(rows)).toEqual({ top: 30, ticks: [0, 6, 12, 18, 24, 30] });
  });
});
