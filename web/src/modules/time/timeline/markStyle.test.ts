import { markStyle } from "./markStyle";

describe("markStyle", () => {
  it("paints an Active Segment in its app colour", () => {
    expect(markStyle("no", "var(--chart-1)", false)).toMatchObject({ background: "var(--chart-1)", opacity: 1 });
  });

  it("hatches Away", () => {
    expect(String(markStyle("yes", "var(--chart-1)", false).backgroundImage)).toContain("repeating-linear-gradient");
  });

  it("fades unknown Away toward the unknown token", () => {
    expect(String(markStyle("unknown", "var(--chart-1)", false).background)).toContain("var(--away-unknown)");
  });

  it("dims what is not highlighted", () => {
    expect(markStyle("no", "var(--chart-1)", true).opacity).toBeLessThan(0.5);
  });
});
