import { formatMask, maskOrNone } from "./mask";

describe("formatMask", () => {
  it("shows every masked number the same way, in capitals", () => {
    expect(formatMask("xxxxxxx1111")).toBe("XXXXXXX1111");
    expect(formatMask(" 4000XXXXXXXX7777 ")).toBe("4000XXXXXXXX7777");
  });
  it("is empty when no number was printed", () => {
    expect(formatMask(null)).toBe("");
    expect(formatMask(undefined)).toBe("");
  });
});

describe("maskOrNone", () => {
  it("says so when no number was printed", () => {
    expect(maskOrNone(null)).toBe("No number printed");
    expect(maskOrNone("xx56")).toBe("XX56");
  });
});
