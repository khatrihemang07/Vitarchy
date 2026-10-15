import { buildNav } from "./nav";
import type { ModuleManifest } from "./module-manifest";

const fake = (id: string, label: string): ModuleManifest => ({
  id,
  label,
  accent: "#fff",
  icon: () => null,
  routes: () => [],
  navItem: { to: `/${id}` },
});

describe("buildNav", () => {
  it("is empty for an empty registry", () => {
    expect(buildNav([])).toEqual([]);
  });

  it("produces one entry per registered Module, in registry order", () => {
    const nav = buildNav([fake("time", "Time"), fake("finance", "Finance")]);
    expect(nav.map((n) => [n.id, n.label, n.to])).toEqual([
      ["time", "Time", "/time"],
      ["finance", "Finance", "/finance"],
    ]);
  });
});
