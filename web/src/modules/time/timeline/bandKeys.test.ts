import { nextMark } from "./bandKeys";

describe("nextMark", () => {
  it("moves one mark with the arrows and stops at the ends", () => {
    expect(nextMark("ArrowRight", 1, 4)).toBe(2);
    expect(nextMark("ArrowLeft", 1, 4)).toBe(0);
    expect(nextMark("ArrowLeft", 0, 4)).toBe(0);
    expect(nextMark("ArrowRight", 3, 4)).toBe(3);
  });
  it("jumps to the first and last mark with Home and End", () => {
    expect(nextMark("Home", 2, 4)).toBe(0);
    expect(nextMark("End", 0, 4)).toBe(3);
  });
  it("ignores other keys, and an empty band", () => {
    expect(nextMark("a", 1, 4)).toBeNull();
    expect(nextMark("ArrowRight", 0, 0)).toBeNull();
  });
});
