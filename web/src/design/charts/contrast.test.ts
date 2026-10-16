import { readFileSync } from "node:fs";
import { join } from "node:path";
import { CHART_SURFACES, contrastRatio, MIN_MARK_CONTRAST, usableSeriesColor } from "./contrast";

const tokens = readFileSync(join(import.meta.dirname, "../theme/tokens.css"), "utf8");
const lightAt = tokens.indexOf('[data-theme="light"] {');
const cardOf = (block: string) => /--card:\s*(#[0-9a-f]{6})/i.exec(block)?.[1];

describe("contrastRatio", () => {
  it("is 21 for black on white and 1 for a colour on itself", () => {
    expect(contrastRatio("#000000", "#ffffff")).toBeCloseTo(21, 1);
    expect(contrastRatio("#2a78d6", "#2a78d6")).toBeCloseTo(1, 5);
  });
});

describe("CHART_SURFACES", () => {
  it("are the card colours of both themes, so the check cannot drift from the tokens", () => {
    expect(CHART_SURFACES).toEqual([cardOf(tokens.slice(lightAt)), cardOf(tokens.slice(0, lightAt))]);
  });
});

describe("usableSeriesColor", () => {
  it("accepts a colour that stands out on both surfaces", () => {
    expect(usableSeriesColor("#2a78d6", [])).toBe("#2a78d6");
    expect(MIN_MARK_CONTRAST).toBeGreaterThanOrEqual(3);
  });

  it("rejects a colour too pale for the light surface or too dark for the dark one", () => {
    expect(usableSeriesColor("#f5e6a8", [])).toBeUndefined();
    expect(usableSeriesColor("#1a1a2a", [])).toBeUndefined();
  });

  it("rejects what is not a six digit hex colour, including nothing at all", () => {
    for (const bad of [undefined, null, "", "blue", "#fff", "url(#x)", "#2a78d6; x"]) expect(usableSeriesColor(bad, [])).toBeUndefined();
  });

  it("rejects a colour too close to one already taken, and reads case-insensitively", () => {
    expect(usableSeriesColor("#2A78D6", ["#2a78d6"])).toBeUndefined();
    expect(usableSeriesColor("#2b79d6", ["#2a78d6"])).toBeUndefined();
    expect(usableSeriesColor("#d95926", ["#2a78d6"])).toBe("#d95926");
  });
});
