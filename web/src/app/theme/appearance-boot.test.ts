import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { APPEARANCE_CACHE_KEY, MOTIONS, THEMES } from "./apply";

const source = readFileSync(resolve(import.meta.dirname, "../../../public/appearance-boot.js"), "utf8");
const root = document.documentElement;

/** Runs the boot script as the browser would: once, before the app loads. */
const boot = () => {
  const script = document.createElement("script");
  script.textContent = source;
  document.head.append(script);
  script.remove();
};

const originalMatchMedia = window.matchMedia.bind(window);

function setSystemDark(dark: boolean) {
  window.matchMedia = ((query: string) => ({
    matches: dark && query.includes("dark"),
  })) as typeof window.matchMedia;
}

beforeEach(() => {
  setSystemDark(false);
});

afterEach(() => {
  localStorage.clear();
  root.removeAttribute("data-theme");
  root.removeAttribute("data-motion");
  root.style.removeProperty("color-scheme");
  window.matchMedia = originalMatchMedia;
  vi.restoreAllMocks();
});

const cache = (value: unknown) => {
  localStorage.setItem(APPEARANCE_CACHE_KEY, JSON.stringify(value));
};

describe("appearance-boot.js", () => {
  it("applies a valid cached theme and motion", () => {
    cache({ theme: "light", motion: "reduced" });
    boot();
    expect(root).toHaveAttribute("data-theme", "light");
    expect(root).toHaveAttribute("data-motion", "reduced");
    expect(root.style.colorScheme).toBe("light");
  });

  it("resolves a cached system theme against the OS", () => {
    setSystemDark(true);
    cache({ theme: "system", motion: "full" });
    boot();
    expect(root).toHaveAttribute("data-theme", "dark");
  });

  it.each([THEMES])("accepts the theme %s that the app can cache", (theme) => {
    cache({ theme, motion: "system" });
    boot();
    expect(root).toHaveAttribute("data-theme", theme === "system" ? "light" : theme);
  });

  it.each([MOTIONS])("accepts the motion %s that the app can cache", (motion) => {
    cache({ theme: "system", motion });
    boot();
    expect(root).toHaveAttribute("data-motion", motion);
  });

  it.each([
    ["unknown values", { theme: "neon", motion: "wild" }],
    ["wrong types", { theme: 3, motion: ["full"] }],
    ["an array", [1, 2]],
    ["null", null],
    ["a string", "dark"],
    ["an inherited key", JSON.parse('{"__proto__":{"theme":"dark"}}') as unknown],
  ])("falls back to following the system for %s", (_name, value) => {
    setSystemDark(true);
    cache(value);
    boot();
    expect(root).toHaveAttribute("data-theme", "dark");
    expect(root).toHaveAttribute("data-motion", "system");
  });

  it("keeps a valid half when the other half is bad", () => {
    cache({ theme: "light", motion: "wild" });
    boot();
    expect(root).toHaveAttribute("data-theme", "light");
    expect(root).toHaveAttribute("data-motion", "system");
  });

  it("falls back when the cache is not JSON", () => {
    localStorage.setItem(APPEARANCE_CACHE_KEY, "{nope");
    boot();
    expect(root).toHaveAttribute("data-theme", "light");
    expect(root).toHaveAttribute("data-motion", "system");
  });

  it("falls back when storage throws", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("blocked");
    });
    boot();
    expect(root).toHaveAttribute("data-theme", "light");
    expect(root).toHaveAttribute("data-motion", "system");
  });

  it("reads the same cache key the app writes", () => {
    const getItem = vi.spyOn(Storage.prototype, "getItem");
    boot();
    expect(getItem).toHaveBeenCalledWith(APPEARANCE_CACHE_KEY);
  });
});
