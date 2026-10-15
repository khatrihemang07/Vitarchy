import { dateFormatLabel } from "../format/datetime";
import {
  appearanceSection,
  coreSections,
  DEFAULT_CORE,
  DISPLAY_NAME_MAX_LENGTH,
  generalSection,
} from "./core-sections";
import { optionValues } from "./schema";

describe("core sections", () => {
  it("derive the defaults of a fresh install", () => {
    expect(DEFAULT_CORE).toEqual({
      display_name: "",
      theme: "system",
      motion: "system",
      week_start: "monday",
      clock: "system",
      date_format: "system",
    });
  });

  it("are scoped to the core settings", () => {
    expect(coreSections.map((s) => s.scope)).toEqual(["core", "core"]);
  });

  it("limit the display name to the server's 50 characters", () => {
    expect(DISPLAY_NAME_MAX_LENGTH).toBe(50);
    const field = generalSection.fields.find((f) => f.key === "display_name");
    expect(field).toMatchObject({ kind: "text", maxLength: 50 });
  });

  it("offer the values the server accepts", () => {
    expect(optionValues(appearanceSection, "theme")).toEqual(["system", "light", "dark"]);
    expect(optionValues(appearanceSection, "motion")).toEqual(["system", "reduced", "full"]);
  });

  it("label each date format with what it produces", () => {
    const field = generalSection.fields.find((f) => f.key === "date_format");
    if (field?.kind !== "select") throw new Error("date_format should be a select");
    expect(field.options.map((o) => o.value)).toEqual([
      "system",
      "day_month_year",
      "month_day_year",
      "iso",
    ]);
    expect(field.options.find((o) => o.value === "iso")?.label).toBe(dateFormatLabel("iso"));
    expect(field.options.find((o) => o.value === "iso")?.label).toBe("2026-10-16");
  });
});
