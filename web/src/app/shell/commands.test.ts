import { Clock, Settings as Gear } from "lucide-react";
import { buildCommandGroups } from "./commands";

const nav = [{ id: "time", label: "Time", to: "/time", accent: "red", icon: Clock }];
const sections = [{ id: "general", scope: "core", label: "General", icon: Gear, fields: [] }];

describe("buildCommandGroups", () => {
  it("lists Home, then each Module, under Go to", () => {
    const [goTo] = buildCommandGroups(nav, sections);
    expect(goTo?.heading).toBe("Go to");
    expect(goTo?.entries.map((e) => [e.label, e.to])).toEqual([
      ["Home", "/"],
      ["Time", "/time"],
    ]);
  });

  it("lists Settings, then each section with a searchable name", () => {
    const [, settings] = buildCommandGroups(nav, sections);
    expect(settings?.entries.map((e) => [e.label, e.to])).toEqual([
      ["Settings", "/settings/general"],
      ["Settings: General", "/settings/general"],
    ]);
    expect(settings?.entries[1]?.searchValue).toBe("Settings General");
  });

  it("still offers Home and Settings with no Modules", () => {
    const groups = buildCommandGroups([], []);
    expect(groups.map((g) => g.entries.length)).toEqual([1, 1]);
  });
});
