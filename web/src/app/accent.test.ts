import { HOME_ACCENT, resolveAccent } from "./accent";

const entries = [
  { id: "time", accent: "#9d6bff" },
  { id: "finance", accent: "#b6f23a" },
];

describe("resolveAccent", () => {
  it("uses the brand accent on Home", () => {
    expect(resolveAccent("/", entries)).toEqual({ moduleId: null, accent: HOME_ACCENT });
  });

  it("uses the module accent for the module root and nested routes", () => {
    expect(resolveAccent("/time", entries)).toEqual({ moduleId: "time", accent: "#9d6bff" });
    expect(resolveAccent("/finance/budgets/2", entries)).toEqual({
      moduleId: "finance",
      accent: "#b6f23a",
    });
  });

  it("does not match on a shared prefix", () => {
    expect(resolveAccent("/timeline", entries).moduleId).toBeNull();
  });

  it("falls back to the brand accent for unknown routes and an empty registry", () => {
    expect(resolveAccent("/nope", entries).accent).toBe(HOME_ACCENT);
    expect(resolveAccent("/time", []).accent).toBe(HOME_ACCENT);
  });
});
