import { formatChange } from "./formatChange";

describe("formatChange", () => {
  it("shows a rise and a fall in time with an arrow", () => {
    expect(formatChange(1500, "seconds")).toBe("↑ 25m");
    expect(formatChange(-3900, "seconds")).toBe("↓ 1h 5m");
  });

  it("shows no change as a dash", () => {
    expect(formatChange(0, "seconds")).toBe("—");
    expect(formatChange(0, "count")).toBe("—");
  });

  it("formats counts as plain numbers", () => {
    expect(formatChange(3, "count")).toBe("↑ 3");
    expect(formatChange(-2, "count")).toBe("↓ 2");
  });
});
