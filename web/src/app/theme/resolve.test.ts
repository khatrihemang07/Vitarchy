import { motionMode, resolveTheme } from "./resolve";

describe("resolveTheme", () => {
  it.each([
    ["light", true, "light"],
    ["dark", false, "dark"],
    ["system", true, "dark"],
    ["system", false, "light"],
  ] as const)("%s with system dark=%s is %s", (setting, systemDark, expected) => {
    expect(resolveTheme(setting, systemDark)).toBe(expected);
  });
});

describe("motionMode", () => {
  it.each([
    ["system", "user"],
    ["reduced", "always"],
    ["full", "never"],
  ] as const)("%s maps to %s", (setting, expected) => {
    expect(motionMode(setting)).toBe(expected);
  });
});
