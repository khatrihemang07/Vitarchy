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

  it("passes a Module's pages through, and gives none when it has no pages", () => {
    const pages = [{ id: "reports", label: "Reports", to: "/time/reports" }];
    const [withPages, without] = buildNav([{ ...fake("time", "Time"), pages }, fake("finance", "Finance")]);
    expect(withPages?.pages).toEqual(pages);
    expect(without?.pages).toEqual([]);
  });
});
