import { changeDirection, changeText } from "./changeText";

describe("changeDirection", () => {
  it("tells the direction", () => {
    expect(changeDirection(5)).toBe("up");
    expect(changeDirection(-5)).toBe("down");
    expect(changeDirection(0)).toBe("none");
  });
});

describe("changeText", () => {
  const amount = (delta: number) => `${String(Math.abs(delta))} units`;
  it("puts an arrow before the size of a rise or a fall", () => {
    expect(changeText(3, amount)).toBe("↑ 3 units");
    expect(changeText(-2, amount)).toBe("↓ 2 units");
  });
  it("shows no change as a dash without asking for an amount", () => {
    expect(changeText(0, () => { throw new Error("not called"); })).toBe("—");
  });
});
