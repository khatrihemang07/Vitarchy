import { skeletonWidth } from "./skeletonWidths";

describe("skeletonWidth", () => {
  it("is the same for the same cell", () => {
    expect(skeletonWidth(3, 1)).toBe(skeletonWidth(3, 1));
  });

  it("varies between neighbouring cells", () => {
    expect(skeletonWidth(0, 0)).not.toBe(skeletonWidth(0, 1));
    expect(skeletonWidth(0, 0)).not.toBe(skeletonWidth(1, 0));
  });
});
