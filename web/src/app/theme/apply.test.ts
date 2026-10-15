import { APPEARANCE_CACHE_KEY, applyAppearance, readCachedAppearance } from "./apply";

const root = document.documentElement;

afterEach(() => {
  localStorage.clear();
  root.removeAttribute("data-theme");
  root.removeAttribute("data-motion");
  root.style.removeProperty("color-scheme");
  vi.restoreAllMocks();
});

describe("applyAppearance", () => {
  it("sets the resolved theme, motion setting and color-scheme on <html>", () => {
    applyAppearance({ theme: "light", motion: "reduced" }, true);
    expect(root).toHaveAttribute("data-theme", "light");
    expect(root).toHaveAttribute("data-motion", "reduced");
    expect(root.style.colorScheme).toBe("light");
  });

  it("resolves system against the OS setting", () => {
    applyAppearance({ theme: "system", motion: "system" }, true);
    expect(root).toHaveAttribute("data-theme", "dark");
    expect(root).toHaveAttribute("data-motion", "system");
  });

  it("caches the raw settings for the boot script", () => {
    applyAppearance({ theme: "system", motion: "full" }, false);
    expect(JSON.parse(localStorage.getItem(APPEARANCE_CACHE_KEY) ?? "null")).toEqual({
      theme: "system",
      motion: "full",
    });
    expect(readCachedAppearance()).toEqual({ theme: "system", motion: "full" });
  });

  it("still applies the attributes when storage throws", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("blocked");
    });
    applyAppearance({ theme: "dark", motion: "system" }, false);
    expect(root).toHaveAttribute("data-theme", "dark");
  });
});

describe("readCachedAppearance", () => {
  it("is undefined for an empty or corrupt cache", () => {
    expect(readCachedAppearance()).toBeUndefined();
    localStorage.setItem(APPEARANCE_CACHE_KEY, "{nope");
    expect(readCachedAppearance()).toBeUndefined();
    localStorage.setItem(APPEARANCE_CACHE_KEY, JSON.stringify({ theme: "neon", motion: "full" }));
    expect(readCachedAppearance()).toBeUndefined();
  });
});

describe("theme swap", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
    root.removeAttribute("data-theme-switching");
  });

  it("pauses transitions while the theme changes, then lets them resume", () => {
    applyAppearance({ theme: "dark", motion: "system" }, false);
    root.removeAttribute("data-theme-switching");

    applyAppearance({ theme: "light", motion: "system" }, false);
    expect(root).toHaveAttribute("data-theme-switching");

    vi.runAllTimers();
    expect(root).not.toHaveAttribute("data-theme-switching");
  });

  it("leaves transitions alone when the resolved theme is unchanged", () => {
    applyAppearance({ theme: "dark", motion: "system" }, false);
    vi.runAllTimers();

    applyAppearance({ theme: "dark", motion: "reduced" }, false);
    expect(root).not.toHaveAttribute("data-theme-switching");
  });
});
