import { parseCssDuration } from "./cssDuration";

describe("parseCssDuration", () => {
  it("reads milliseconds and seconds", () => {
    expect(parseCssDuration("250ms")).toBe(250);
    expect(parseCssDuration(" 0.25s ")).toBe(250);
    expect(parseCssDuration("2s")).toBe(2000);
  });

  it("is 0 for an empty or unreadable value", () => {
    expect(parseCssDuration("")).toBe(0);
    expect(parseCssDuration("soon")).toBe(0);
    expect(parseCssDuration("250")).toBe(0);
  });
});
