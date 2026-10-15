import { streakLabel, streakTier } from "./streak";

describe("streakTier", () => {
  it.each([
    [0, "cold"],
    [-3, "cold"],
    [1, "warm"],
    [6, "warm"],
    [7, "hot"],
    [30, "hot"],
  ] as const)("%i days is %s", (days, tier) => {
    expect(streakTier(days)).toBe(tier);
  });
});

describe("streakLabel", () => {
  it("says there is no streak when cold", () => {
    expect(streakLabel(0)).toBe("No streak yet");
  });

  it("counts days when warm or hot", () => {
    expect(streakLabel(1)).toBe("1-day streak");
    expect(streakLabel(9)).toBe("9-day streak");
  });
});
