import { assignProjectColors } from "./projectColors";

const project = (name: string, color: string | null) => ({ name, color });

describe("assignProjectColors", () => {
  it("keeps a project's own Toggl colour when it reads well on both themes", () => {
    const colors = assignProjectColors([project("office", "#2a78d6"), project("work", "#d95926")]);
    expect(colors.get("office")).toBe("#2a78d6");
    expect(colors.get("work")).toBe("#d95926");
  });

  it("uses the palette for every project when any Toggl colour is unreadable or missing, so the two never clash", () => {
    const colors = assignProjectColors([project("sleep", "#fdf6c4"), project("body", null), project("office", "#2a78d6")]);
    expect([...colors.values()]).toEqual(["var(--chart-1)", "var(--chart-2)", "var(--chart-3)"]);
  });

  it("uses the palette when two projects' colours would be mistaken for each other", () => {
    const colors = assignProjectColors([project("a", "#2a78d6"), project("b", "#2b79d7")]);
    expect([...colors.values()]).toEqual(["var(--chart-1)", "var(--chart-2)"]);
  });

  it("folds projects past the palette into the neutral colour", () => {
    const many = Array.from({ length: 10 }, (_, i) => project(`p${String(i)}`, null));
    const colors = assignProjectColors(many);
    expect(colors.get("p7")).toBe("var(--chart-8)");
    expect(colors.get("p8")).toBe("var(--chart-other)");
  });
});
