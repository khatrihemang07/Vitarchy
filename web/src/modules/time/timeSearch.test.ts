import { resolveView, parseTimeSearch, toSearch } from "./timeSearch";

describe("parseTimeSearch", () => {
  it("keeps valid values", () => {
    expect(parseTimeSearch({ tab: "activity", period: "month", from: "2026-10-01" })).toEqual({
      tab: "activity",
      period: "month",
      from: "2026-10-01",
    });
  });

  it("drops anything unknown or malformed instead of failing", () => {
    expect(parseTimeSearch({ tab: "nope", period: "decade", from: "2026-02-30", to: 7 })).toEqual({});
  });

  it("keeps an end date only for a custom range that is not reversed", () => {
    expect(parseTimeSearch({ period: "custom", from: "2026-10-01", to: "2026-10-09" })).toMatchObject({ to: "2026-10-09" });
    expect(parseTimeSearch({ period: "custom", from: "2026-10-09", to: "2026-10-01" })).not.toHaveProperty("to");
    expect(parseTimeSearch({ period: "week", from: "2026-10-01", to: "2026-10-09" })).not.toHaveProperty("to");
  });
});

describe("resolveView", () => {
  it("defaults to the Overview of the current week", () => {
    expect(resolveView({}, "2026-10-14", "monday")).toEqual({
      tab: "overview",
      selection: { kind: "week", from: "2026-10-12", to: "2026-10-18" },
    });
  });

  it("snaps a stored week to the week-start Setting", () => {
    const view = resolveView({ period: "week", from: "2026-10-14" }, "2026-10-20", "sunday");
    expect(view.selection).toEqual({ kind: "week", from: "2026-10-11", to: "2026-10-17" });
  });

  it("keeps a custom range as written", () => {
    const view = resolveView({ period: "custom", from: "2026-10-01", to: "2026-10-09" }, "2026-10-20", "monday");
    expect(view.selection).toEqual({ kind: "custom", from: "2026-10-01", to: "2026-10-09" });
  });
});

describe("toSearch", () => {
  it("round-trips through the URL form", () => {
    const view = { tab: "life-log", selection: { kind: "custom", from: "2026-10-01", to: "2026-10-09" } } as const;
    expect(resolveView(parseTimeSearch(toSearch(view.tab, view.selection)), "2026-10-20", "monday")).toEqual(view);
  });

  it("leaves out the end date of a period that follows from its start", () => {
    expect(toSearch("overview", { kind: "week", from: "2026-10-12", to: "2026-10-18" })).toEqual({
      tab: "overview",
      period: "week",
      from: "2026-10-12",
    });
  });
});
