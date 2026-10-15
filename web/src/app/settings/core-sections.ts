import { Palette, SlidersHorizontal } from "lucide-react";
import { CORE_SCOPE, type CoreSettings } from "@/api/settings";
import { dateFormatLabel } from "../format/datetime";
import { type SettingsSection, withDefaults } from "./schema";

/** Longest display name, in Unicode code points. Mirrors the server's limit. */
export const DISPLAY_NAME_MAX_LENGTH = 50;

type DateFormat = CoreSettings["date_format"];
const DATE_FORMATS: readonly DateFormat[] = ["system", "day_month_year", "month_day_year", "iso"];

/** Keys and option values mirror the backend `CoreSettings` (snake_case enums). */
export const generalSection: SettingsSection = {
  id: "general",
  scope: CORE_SCOPE,
  label: "General",
  icon: SlidersHorizontal,
  fields: [
    {
      kind: "text",
      key: "display_name",
      label: "Display name",
      description: "How Vitarchy greets you.",
      placeholder: "Your name",
      default: "",
      maxLength: DISPLAY_NAME_MAX_LENGTH,
    },
    {
      kind: "select",
      key: "week_start",
      label: "Week starts on",
      default: "monday",
      options: [
        { value: "monday", label: "Monday" },
        { value: "sunday", label: "Sunday" },
      ],
    },
    {
      kind: "select",
      key: "clock",
      label: "Clock",
      default: "system",
      options: [
        { value: "system", label: "System" },
        { value: "h12", label: "12-hour" },
        { value: "h24", label: "24-hour" },
      ],
    },
    {
      kind: "select",
      key: "date_format",
      label: "Date format",
      default: "system",
      options: DATE_FORMATS.map((value) => ({ value, label: dateFormatLabel(value) })),
    },
  ],
};

export const appearanceSection: SettingsSection = {
  id: "appearance",
  scope: CORE_SCOPE,
  label: "Appearance",
  icon: Palette,
  fields: [
    {
      kind: "select",
      key: "theme",
      label: "Theme",
      description: "System follows your device.",
      default: "system",
      options: [
        { value: "system", label: "System" },
        { value: "light", label: "Light" },
        { value: "dark", label: "Dark" },
      ],
    },
    {
      kind: "select",
      key: "motion",
      label: "Motion",
      description: "Reduced stops springs and decorative animation.",
      default: "system",
      options: [
        { value: "system", label: "System" },
        { value: "reduced", label: "Reduced" },
        { value: "full", label: "Full" },
      ],
    },
  ],
};

export const coreSections: readonly SettingsSection[] = [generalSection, appearanceSection];

/**
 * The core settings of a fresh install, derived from the declared defaults so
 * they cannot drift. Used until the server answers.
 */
export const DEFAULT_CORE = withDefaults(
  { fields: coreSections.flatMap((section) => section.fields) },
  undefined,
) as unknown as CoreSettings;
