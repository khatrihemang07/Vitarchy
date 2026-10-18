import { appColorMap, colorOf, OTHER_COLOR } from "./appColors";

const apps = ["Code", "Safari", "Slack", "Mail", "Notes", "Figma", "Terminal", "Zoom", "Preview", "Finder"].map((app) => ({ app }));

describe("appColorMap", () => {
  it("gives the busiest apps the chart colours in rank order", () => {
    const colors = appColorMap(apps);
    expect(colorOf(colors, "Code")).toBe("var(--chart-1)");
    expect(colorOf(colors, "Safari")).toBe("var(--chart-2)");
    expect(colorOf(colors, "Zoom")).toBe("var(--chart-8)");
  });

  it("puts apps past the palette, and apps it has never seen, in Other", () => {
    const colors = appColorMap(apps);
    expect(colorOf(colors, "Preview")).toBe(OTHER_COLOR);
    expect(colorOf(colors, "Finder")).toBe(OTHER_COLOR);
    expect(colorOf(colors, "Unlisted")).toBe(OTHER_COLOR);
  });

  it("is the same for the same response, whichever consumer asks", () => {
    expect([...appColorMap(apps)]).toEqual([...appColorMap(apps)]);
  });

  it("copes with no apps", () => {
    expect(colorOf(appColorMap([]), "Code")).toBe(OTHER_COLOR);
  });
});
