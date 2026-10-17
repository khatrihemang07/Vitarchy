import { loadLastFilters, saveLastFilters } from "./lastFilters";

afterEach(() => {
  localStorage.clear();
  vi.restoreAllMocks();
});

describe("lastFilters", () => {
  it("is null before anything was saved", () => {
    expect(loadLastFilters()).toBeNull();
  });

  it("remembers what was saved", () => {
    saveLastFilters({ project: [1, 2], tag: ["a"], client: [] });
    expect(loadLastFilters()).toEqual({ project: [1, 2], tag: ["a"], client: [] });
  });

  it("remembers an empty choice, which differs from never having chosen", () => {
    saveLastFilters({ project: [], tag: [], client: [] });
    expect(loadLastFilters()).toEqual({ project: [], tag: [], client: [] });
  });

  it("ignores damaged storage", () => {
    localStorage.setItem("vitarchy.time.reports.filters", "{nope");
    expect(loadLastFilters()).toBeNull();
    localStorage.setItem("vitarchy.time.reports.filters", JSON.stringify({ project: "x", tag: [1, "ok"], client: [3, -1] }));
    expect(loadLastFilters()).toEqual({ project: [], tag: ["1", "ok"], client: [3] });
  });

  it("survives storage that throws", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("blocked");
    });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("blocked");
    });
    expect(loadLastFilters()).toBeNull();
    expect(() => {
      saveLastFilters({ project: [1], tag: [], client: [] });
    }).not.toThrow();
  });
});
