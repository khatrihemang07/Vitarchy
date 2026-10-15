import { parseNumber } from "./parseNumber";

describe("parseNumber", () => {
  it("parses a plain number", () => {
    expect(parseNumber("7")).toBe(7);
    expect(parseNumber(" 2.5 ")).toBe(2.5);
  });

  it("clamps to the bounds", () => {
    expect(parseNumber("99", 1, 10)).toBe(10);
    expect(parseNumber("-4", 1, 10)).toBe(1);
  });

  it("is undefined for blank or non-numeric text", () => {
    expect(parseNumber("")).toBeUndefined();
    expect(parseNumber("   ")).toBeUndefined();
    expect(parseNumber("abc")).toBeUndefined();
    expect(parseNumber("Infinity")).toBeUndefined();
  });
});
