import { swatchStyle } from "./stackSeries";

describe("swatchStyle", () => {
  it("fills a solid series and hatches or dashes the others so they read apart without colour", () => {
    expect(swatchStyle({ look: "solid", color: "red" })).toEqual({ background: "red" });
    expect(swatchStyle({ look: "hatched", color: "red" })).toMatchObject({ backgroundImage: expect.stringContaining("repeating-linear-gradient") as string });
    expect(swatchStyle({ look: "faded", color: "red" })).toMatchObject({ border: "1px dashed red" });
  });

  it("outlines a faded swatch in its own border colour when it has one", () => {
    expect(swatchStyle({ look: "faded", color: "grey", border: "red" })).toMatchObject({ border: "1px dashed red" });
  });
});
