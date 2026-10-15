import { Puzzle } from "lucide-react";
import type { ModuleManifest } from "../module-manifest";
import { buildSections } from "./sections";

const fake = (id: string, settings?: ModuleManifest["settings"]): ModuleManifest => ({
  id,
  label: id,
  accent: "#fff",
  icon: () => null,
  routes: () => [],
  navItem: { to: `/${id}` },
  ...(settings && { settings }),
});

describe("buildSections", () => {
  it("lists the core sections first when there are no Modules", () => {
    expect(buildSections([]).map((s) => s.id)).toEqual(["general", "appearance"]);
  });

  it("adds a section, keyed by Module id, for each Module that declares settings", () => {
    const sections = buildSections([
      fake("time", { label: "Time", icon: Puzzle, fields: [] }),
      fake("quiet"),
    ]);
    expect(sections.map((s) => s.id)).toEqual(["general", "appearance", "time"]);
    expect(sections[2]?.label).toBe("Time");
    expect(sections.map((s) => s.scope)).toEqual(["core", "core", "time"]);
  });
});
