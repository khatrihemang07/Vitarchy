import { optionValues, withDefaults, type SettingsSection } from "./schema";

const section: SettingsSection = {
  id: "demo",
  scope: "demo",
  label: "Demo",
  icon: () => null,
  fields: [
    {
      kind: "select",
      key: "mode",
      label: "Mode",
      default: "a",
      options: [
        { value: "a", label: "A" },
        { value: "b", label: "B" },
      ],
    },
    { kind: "toggle", key: "on", label: "On", default: true },
    { kind: "text", key: "name", label: "Name", default: "" },
    { kind: "number", key: "count", label: "Count", default: 3 },
  ],
};

describe("withDefaults", () => {
  it("returns every default when nothing is stored", () => {
    expect(withDefaults(section, undefined)).toEqual({ mode: "a", on: true, name: "", count: 3 });
  });

  it("lets stored values win over defaults", () => {
    const stored = { mode: "b", on: false, name: "Ada", count: 9 };
    expect(withDefaults(section, stored)).toEqual(stored);
  });

  it("fills only the missing keys", () => {
    expect(withDefaults(section, { name: "Ada" })).toEqual({
      mode: "a",
      on: true,
      name: "Ada",
      count: 3,
    });
  });

  it("ignores unknown keys", () => {
    expect(withDefaults(section, { nope: 1 })).not.toHaveProperty("nope");
  });

  it("falls back to the default for a value of the wrong type or an unknown option", () => {
    expect(withDefaults(section, { mode: "zzz", on: "yes", name: 4, count: "x" })).toEqual({
      mode: "a",
      on: true,
      name: "",
      count: 3,
    });
  });
});

describe("optionValues", () => {
  it("lists a select field's values", () => {
    expect(optionValues(section, "mode")).toEqual(["a", "b"]);
  });

  it("is empty for other kinds and unknown keys", () => {
    expect(optionValues(section, "on")).toEqual([]);
    expect(optionValues(section, "missing")).toEqual([]);
  });
});
