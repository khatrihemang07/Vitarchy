import { greeting, greetingFor } from "./greeting";

const at = (hour: number) => new Date(2026, 9, 16, hour, 30);

describe("greetingFor", () => {
  it.each([
    [0, "Good evening"],
    [4, "Good evening"],
    [5, "Good morning"],
    [11, "Good morning"],
    [12, "Good afternoon"],
    [17, "Good afternoon"],
    [18, "Good evening"],
    [23, "Good evening"],
  ])("at %i:30 says %s", (hour, expected) => {
    expect(greetingFor(at(hour))).toBe(expected);
  });
});

describe("greeting", () => {
  it("adds the display name", () => {
    expect(greeting(at(20), "Hemang")).toBe("Good evening, Hemang");
  });

  it.each(["", "   "])("omits a blank name (%j)", (name) => {
    expect(greeting(at(8), name)).toBe("Good morning");
  });

  it("trims the name", () => {
    expect(greeting(at(14), "  Ada ")).toBe("Good afternoon, Ada");
  });
});
