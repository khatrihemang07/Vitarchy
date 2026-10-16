import { readFileSync } from "node:fs";
import { join } from "node:path";
import { OTHER_COLOR, SERIES_COUNT, seriesColor } from "./palette";

const tokens = readFileSync(join(import.meta.dirname, "../theme/tokens.css"), "utf8");
const darkBlock = tokens.slice(0, tokens.indexOf('[data-theme="light"] {'));
const lightBlock = tokens.slice(tokens.indexOf('[data-theme="light"] {'), tokens.indexOf("@theme inline"));

describe("seriesColor", () => {
  it("gives each series its own colour variable, in order", () => {
    const colors = Array.from({ length: SERIES_COUNT }, (_, i) => seriesColor(i));
    expect(colors[0]).toBe("var(--chart-1)");
    expect(new Set(colors).size).toBe(SERIES_COUNT);
  });

  it("folds series past the palette into the neutral Other colour, never a new hue", () => {
    expect(seriesColor(SERIES_COUNT)).toBe("var(--chart-other)");
    expect(seriesColor(40)).toBe(OTHER_COLOR);
  });
});

describe("chart tokens", () => {
  it("defines every series colour in both themes", () => {
    const names = [...Array.from({ length: SERIES_COUNT }, (_, i) => `--chart-${String(i + 1)}`), "--chart-other"];
    for (const name of names) {
      expect(darkBlock).toContain(`${name}:`);
      expect(lightBlock).toContain(`${name}:`);
    }
  });
});
