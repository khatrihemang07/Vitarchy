import { Palette, SlidersHorizontal } from "lucide-react";
import { CORE_CHOICES, CORE_SCOPE, DEFAULT_CORE } from "@/api/settings";
import { dateFormatLabel } from "../format/datetime";
import type { SettingsSection } from "./schema";

/** Longest display name, in Unicode code points. Mirrors the server's limit. */
export const DISPLAY_NAME_MAX_LENGTH = 50;

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
      default: DEFAULT_CORE.display_name,
      maxLength: DISPLAY_NAME_MAX_LENGTH,
    },
    {
      kind: "select",
      key: "week_start",
      label: "Week starts on",
      default: DEFAULT_CORE.week_start,
      options: [
        { value: "monday", label: "Monday" },
        { value: "sunday", label: "Sunday" },
      ],
    },
    {
      kind: "select",
      key: "clock",
      label: "Clock",
      default: DEFAULT_CORE.clock,
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
      default: DEFAULT_CORE.date_format,
      options: CORE_CHOICES.date_format.map((value) => ({ value, label: dateFormatLabel(value) })),
    },
  ],
};

/** The Appearance section of the Core settings. */
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
      default: DEFAULT_CORE.theme,
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
      default: DEFAULT_CORE.motion,
      options: [
        { value: "system", label: "System" },
        { value: "reduced", label: "Reduced" },
        { value: "full", label: "Full" },
      ],
    },
  ],
};

/** Every Core settings section, in page order. */
export const coreSections: readonly SettingsSection[] = [generalSection, appearanceSection];
