import { CORE_SCOPE, type AllSettings, storedFor, withScope } from "./settingsScope";

const all: AllSettings = {
  core: {
    display_name: "Ada",
    theme: "dark",
    motion: "system",
    week_start: "monday",
    clock: "system",
    date_format: "system",
  },
  modules: { time: { goal: 2 } },
};

describe("storedFor", () => {
  it("returns a copy of the core settings for the core scope", () => {
    const stored = storedFor(all, CORE_SCOPE);
    expect(stored).toEqual(all.core);
    expect(stored).not.toBe(all.core);
  });

  it("returns a Module's saved object, or an empty one if nothing is saved", () => {
    expect(storedFor(all, "time")).toEqual({ goal: 2 });
    expect(storedFor(all, "finance")).toEqual({});
  });
});

describe("withScope", () => {
  it("replaces the core settings without touching Modules", () => {
    const next = withScope(all, CORE_SCOPE, { ...all.core, theme: "light" });
    expect(next.core.theme).toBe("light");
    expect(next.modules).toBe(all.modules);
  });

  it("adds or replaces one Module's object and leaves the rest", () => {
    const next = withScope(all, "finance", { currency: "GBP" });
    expect(next.modules).toEqual({ time: { goal: 2 }, finance: { currency: "GBP" } });
    expect(next.core).toBe(all.core);
  });
});
