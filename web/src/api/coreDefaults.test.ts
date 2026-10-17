import { coreWithDefaults, DEFAULT_CORE } from "./coreDefaults";

describe("DEFAULT_CORE", () => {
  it("matches the server's defaults for a fresh install", () => {
    expect(DEFAULT_CORE).toEqual({
      display_name: "",
      theme: "system",
      motion: "system",
      week_start: "monday",
      clock: "system",
      date_format: "system",
    });
  });
});

describe("coreWithDefaults", () => {
  it("returns the defaults when nothing was loaded", () => {
    expect(coreWithDefaults(undefined)).toEqual(DEFAULT_CORE);
  });

  it("keeps every field the server sent", () => {
    const sent = { display_name: "Ana", theme: "dark", motion: "reduced", week_start: "sunday", clock: "h24", date_format: "iso" };
    expect(coreWithDefaults(sent as never)).toEqual(sent);
  });

  it("fills a missing field with its default and keeps the others", () => {
    expect(coreWithDefaults({ week_start: "sunday" } as never)).toEqual({ ...DEFAULT_CORE, week_start: "sunday" });
  });

  it("replaces a field of the wrong type with its default", () => {
    expect(coreWithDefaults({ week_start: 3, clock: null, theme: "dark" } as never)).toEqual({ ...DEFAULT_CORE, theme: "dark" });
  });

  it("replaces a value outside the field's allowed choices with its default, field by field", () => {
    const sent = { week_start: "foo", theme: "dark", motion: "off", clock: "h24", date_format: "" };
    expect(coreWithDefaults(sent as never)).toEqual({ ...DEFAULT_CORE, theme: "dark", clock: "h24" });
  });

  it("keeps any text as a display name", () => {
    expect(coreWithDefaults({ display_name: "Ana Maria" }).display_name).toBe("Ana Maria");
  });

  it("survives a body that is not an object", () => {
    expect(coreWithDefaults("oops" as never)).toEqual(DEFAULT_CORE);
    expect(coreWithDefaults(null)).toEqual(DEFAULT_CORE);
  });

  it("ignores fields it does not know", () => {
    expect(coreWithDefaults({ future: 1 } as never)).toEqual(DEFAULT_CORE);
  });
});
