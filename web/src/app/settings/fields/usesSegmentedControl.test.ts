import { usesSegmentedControl } from "./usesSegmentedControl";

const options = (...labels: string[]) => labels.map((label) => ({ value: label, label }));

describe("usesSegmentedControl", () => {
  it("is true for a few short options", () => {
    expect(usesSegmentedControl(options("System", "Light", "Dark"))).toBe(true);
  });

  it("is false for many options", () => {
    expect(usesSegmentedControl(options("a", "b", "c", "d", "e"))).toBe(false);
  });

  it("is false when the labels are too long to sit side by side", () => {
    expect(
      usesSegmentedControl(
        options("Friday, 16 October 2026", "Friday, October 16, 2026", "2026-10-16"),
      ),
    ).toBe(false);
  });
});
