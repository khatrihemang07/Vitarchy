import { tableLayout } from "./columns";
import { figuresFor } from "./groupFigures";
import { group } from "./testReport";

const ctx = { periodDays: 7 };
const headers = (...args: Parameters<typeof tableLayout>) => tableLayout(...args).columns.map((c) => c.label);

describe("tableLayout", () => {
  it("names the per-Day columns Avg, Min and Max per Day", () => {
    expect(headers("week", "per_day", "previous")).toEqual(["Days", "Entries", "Total", "Avg per Day", "Min per Day", "Max per Day"]);
  });

  it("names the per-entry columns Avg, Min and Max per entry", () => {
    expect(headers("month", "per_entry", "usual")).toEqual(["Days", "Entries", "Total", "Avg per entry", "Min per entry", "Max per entry"]);
  });

  it("drops the figures that only repeat the total on a Day, and adds the Usual against the last 4 weeks", () => {
    expect(headers("day", "per_day", "usual")).toEqual(["Entries", "Total", "Usual", "Usual range"]);
    expect(headers("day", "per_entry", "usual")).toEqual(["Entries", "Total", "Usual", "Usual range"]);
  });

  it("shows only Entries and Total on a Day against the Previous Period", () => {
    expect(headers("day", "per_day", "previous")).toEqual(["Entries", "Total"]);
  });

  it("treats a custom stretch like a week or month", () => {
    expect(headers("custom", "per_day", "previous")).toEqual(headers("week", "per_day", "previous"));
  });

  it("gives every column a tooltip that explains it", () => {
    for (const kind of ["day", "week"] as const) {
      for (const mode of ["per_day", "per_entry"] as const) {
        for (const compare of ["previous", "usual"] as const) {
          for (const column of tableLayout(kind, mode, compare).columns) expect(column.tooltip.length).toBeGreaterThan(10);
        }
      }
    }
    const [, , , , min] = tableLayout("week", "per_day", "previous").columns;
    expect(min?.tooltip).toBe("The smallest daily total among the Days this group happened");
  });

  it("gives the columns of every layout their own id, for React keys and tests", () => {
    for (const kind of ["day", "week"] as const) {
      for (const mode of ["per_day", "per_entry"] as const) {
        for (const compare of ["previous", "usual"] as const) {
          const ids = tableLayout(kind, mode, compare).columns.map((column) => column.id);
          expect(new Set(ids).size).toBe(ids.length);
        }
      }
    }
    expect(tableLayout("day", "per_day", "usual").columns.map((c) => c.id)).toContain("day-total");
  });

  it("has one grid track per column plus the group label", () => {
    const tracks = (grid: string) => /grid-cols-\[(.*?)\]/.exec(grid.split(" ").find((c) => c.startsWith("md:grid-cols-")) ?? "")?.[1];
    expect(tracks(tableLayout("day", "per_day", "usual").grid)?.split("_").length).toBe(5);
    expect(tracks(tableLayout("day", "per_day", "previous").grid)?.split("_").length).toBe(3);
  });
});

describe("column cells", () => {
  const cellsOf = (kind: Parameters<typeof tableLayout>[0], mode: Parameters<typeof tableLayout>[1], compare: Parameters<typeof tableLayout>[2], g = group("Standup")) => {
    const figures = figuresFor(g, mode, compare);
    return tableLayout(kind, mode, compare).columns.map((c) => [c.label, c.cell(figures, ctx)] as const);
  };

  it("shows Days as 'n of N' and the plain figures with their changes", () => {
    const cells = Object.fromEntries(cellsOf("week", "per_day", "previous"));
    expect(cells.Days?.text).toBe("1 of 7");
    expect(cells.Total).toMatchObject({ text: "1h", change: 900, emphasis: true });
    expect(cells["Max per Day"]).toMatchObject({ text: "2h", change: 0 });
  });

  it("shows a Day's total against the Usual's average per Day, with Usual and its range", () => {
    const seed = group("Standup", {
      stats: { days: 1, entries: 2, total_s: 23280, per_day: { avg_s: 23280, min_s: 23280, max_s: 23280 }, per_entry: { avg_s: 1, min_s: 1, max_s: 1 } },
      vs_usual: {
        is_new: false,
        change: { days: 0, entries: 0, total_s: 99999, per_day: { avg_s: -2520, min_s: 0, max_s: 0 }, per_entry: { avg_s: 0, min_s: 0, max_s: 0 } },
        usual: { days: 9, per_day: { avg_s: 25800, min_s: 18000, max_s: 32400 }, per_entry: { avg_s: 1, min_s: 1, max_s: 1 } },
      },
    });
    const cells = Object.fromEntries(cellsOf("day", "per_day", "usual", seed));
    expect(cells.Total).toMatchObject({ text: "6h 28m", change: -2520, emphasis: true });
    expect(cells.Usual).toMatchObject({ text: "7h 10m", change: null });
    expect(cells["Usual range"]).toMatchObject({ text: "5h – 9h" });
  });

  it("shows a single value for the Usual's range when its ends are equal", () => {
    const flat = group("Standup", {
      vs_usual: { is_new: false, change: null, usual: { days: 3, per_day: { avg_s: 2700, min_s: 2700, max_s: 2700 }, per_entry: { avg_s: 1, min_s: 1, max_s: 1 } } },
    });
    expect(Object.fromEntries(cellsOf("day", "per_day", "usual", flat))["Usual range"]).toMatchObject({ text: "45m" });
  });

  it("leaves Usual and its range as a dash for a group missing from the last 4 weeks", () => {
    const fresh = group("Standup", { vs_usual: { is_new: true, change: null } });
    const cells = Object.fromEntries(cellsOf("day", "per_day", "usual", fresh));
    expect(cells.Usual?.text).toBe("—");
    expect(cells["Usual range"]?.text).toBe("—");
    expect(cells.Total?.change).toBeNull();
  });

  it("shows a Day's change from yesterday in Previous Period mode", () => {
    const cells = Object.fromEntries(cellsOf("day", "per_day", "previous"));
    expect(cells.Total?.change).toBe(900);
  });
});
