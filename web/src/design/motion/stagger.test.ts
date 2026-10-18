import { stagger } from "./stagger";

describe("stagger", () => {
  it("starts the first section at once and delays each later one more", () => {
    expect(stagger(0).animationDelay).toBe("0ms");
    expect(stagger(2).animationDelay).toBe("80ms");
  });

  it("never delays by a negative amount", () => {
    expect(stagger(-3).animationDelay).toBe("0ms");
  });
});
