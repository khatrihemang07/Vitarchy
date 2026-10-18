import { parseTimelineSearch, timelineParams, withApp, withPeriod } from "./timelineSearch";

describe("parseTimelineSearch", () => {
  it("opens on the current week with no highlight when the URL says nothing", () => {
    expect(parseTimelineSearch({})).toEqual({ kind: "week" });
  });

  it("keeps a valid Day Period and the highlighted app", () => {
    expect(parseTimelineSearch({ kind: "day", date: "2026-10-16", app: "Safari" })).toEqual({ kind: "day", date: "2026-10-16", app: "Safari" });
  });

  it("keeps a custom range and drops a bad one", () => {
    expect(parseTimelineSearch({ kind: "custom", from: "2026-10-01", to: "2026-10-09" })).toMatchObject({ kind: "custom", from: "2026-10-01", to: "2026-10-09" });
    expect(parseTimelineSearch({ kind: "custom", from: "nope", to: "2026-10-09" })).toEqual({ kind: "week" });
  });

  it("ignores an empty or non-text app and caps a long one", () => {
    expect(parseTimelineSearch({ app: "" })).toEqual({ kind: "week" });
    expect(parseTimelineSearch({ app: { a: 1 } })).toEqual({ kind: "week" });
    expect(parseTimelineSearch({ app: "x".repeat(500) }).app).toHaveLength(200);
  });
});

describe("timelineParams", () => {
  it("leaves defaults out so the address stays short", () => {
    expect(timelineParams({ kind: "week" })).toEqual({});
    expect(timelineParams({ kind: "month", app: "Code" })).toEqual({ kind: "month", app: "Code" });
  });
});

describe("withPeriod and withApp", () => {
  it("clears the highlight when the Period changes, since the apps differ", () => {
    expect(withPeriod({ kind: "day", date: "2026-10-16", app: "Safari" }, "week", { from: "2026-10-12", to: "2026-10-18" })).toEqual({ kind: "week", date: "2026-10-12" });
  });

  it("toggles the highlight: choosing the highlighted app again removes it", () => {
    expect(withApp({ kind: "week" }, "Safari").app).toBe("Safari");
    expect(withApp({ kind: "week", app: "Safari" }, "Safari").app).toBeUndefined();
    expect(withApp({ kind: "week", app: "Safari" }, undefined).app).toBeUndefined();
  });
});
