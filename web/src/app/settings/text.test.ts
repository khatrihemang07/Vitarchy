import { codePointLength, normalizeText } from "./text";

describe("codePointLength", () => {
  it("counts Unicode code points like Rust's chars().count()", () => {
    expect(codePointLength("Ada")).toBe(3);
    expect(codePointLength("😀😀")).toBe(2);
    expect("😀😀".length).toBe(4);
  });
});

describe("normalizeText", () => {
  it("trims the whitespace Rust trims", () => {
    expect(normalizeText("  Ada　\u0085\n", 50)).toBe("Ada");
  });

  it("does not trim the byte-order mark, which Rust keeps", () => {
    expect(normalizeText("﻿Ada", 50)).toBe("﻿Ada");
  });

  it("cuts to the maximum in code points, never splitting an emoji", () => {
    expect(normalizeText("😀😀😀", 2)).toBe("😀😀");
    expect(normalizeText("abcdefgh", 5)).toBe("abcde");
  });

  it("applies no limit when none is given", () => {
    expect(normalizeText("x".repeat(500))).toHaveLength(500);
  });

  it("trims before counting, so padding never eats the budget", () => {
    expect(normalizeText("  abc  ", 3)).toBe("abc");
  });
});
