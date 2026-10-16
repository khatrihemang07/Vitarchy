import { assignProjectColors } from "./projectColors";

const project = (name: string, color: string | null) => ({ name, color });

describe("assignProjectColors", () => {
  it("keeps a project's own Toggl colour when it reads well on both themes", () => {
    const colors = assignProjectColors([project("office", "#2a78d6"), project("work", "#d95926")]);
    expect(colors.get("office")).toBe("#2a78d6");
    expect(colors.get("work")).toBe("#d95926");
  });

  it("gives only the project without a usable colour a palette colour, and not one that looks like a colour in use", () => {
    const colors = assignProjectColors([project("sleep", "#fdf6c4"), project("office", "#2a78d6"), project("work", "#eb6834")]);
    expect(colors.get("office")).toBe("#2a78d6");
    expect(colors.get("work")).toBe("#eb6834");
    // chart-1 is #2a78d6 and chart-2 #eb6834 in the light theme, so the first free slot is chart-3.
    expect(colors.get("sleep")).toBe("var(--chart-3)");
  });

  it("keeps the first of two look-alike colours and gives the second a palette colour", () => {
    const colors = assignProjectColors([project("a", "#2a78d6"), project("b", "#2b79d7")]);
    expect(colors.get("a")).toBe("#2a78d6");
    expect(colors.get("b")).toBe("var(--chart-2)");
  });

  it("paints the No project bucket in the neutral colour and lets it neither spoil nor take a colour", () => {
    const colors = assignProjectColors([project("No project", null), project("office", "#2a78d6"), project("body", null)]);
    expect(colors.get("No project")).toBe("var(--chart-other)");
    expect(colors.get("office")).toBe("#2a78d6");
    expect(colors.get("body")).toBe("var(--chart-2)");
  });

  it("folds projects past the palette into the neutral colour", () => {
    const many = Array.from({ length: 10 }, (_, i) => project(`p${String(i)}`, null));
    const colors = assignProjectColors(many);
    expect(colors.get("p7")).toBe("var(--chart-8)");
    expect(colors.get("p8")).toBe("var(--chart-other)");
  });
});
