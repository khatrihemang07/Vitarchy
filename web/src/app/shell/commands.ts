import type { ComponentType } from "react";
import { HOME_NAV, type NavEntry, SETTINGS_NAV } from "../nav";
import type { SettingsSection } from "../settings/schema";

/** One jump target in the command bar. */
export interface CommandEntry {
  /** Stable React key; unique across all groups. */
  key: string;
  /** Text shown for the entry. */
  label: string;
  /** Route the entry jumps to. */
  to: string;
  /** Icon drawn beside the label. */
  icon: ComponentType<{ className?: string }>;
  /** What the search matches on, when it should differ from the label. */
  searchValue?: string;
}

/** A headed list of entries. */
export interface CommandGroup {
  /** Title shown above the group. */
  heading: string;
  /** The group's entries, in display order. */
  entries: CommandEntry[];
}

/** The command bar's jump targets: every screen, then every Settings section. */
export function buildCommandGroups(
  nav: readonly NavEntry[],
  sections: readonly SettingsSection[],
): CommandGroup[] {
  return [
    {
      heading: "Go to",
      entries: [
        { key: "home", ...HOME_NAV },
        ...nav.flatMap(({ id, label, to, icon, pages }) => [
          { key: id, label, to, icon },
          ...pages.map((page) => ({
            key: `${id}-${page.id}`,
            label: `${label}: ${page.label}`,
            searchValue: `${label} ${page.label}`,
            to: page.to,
            icon: page.icon ?? icon,
          })),
        ]),
      ],
    },
    {
      heading: "Settings",
      entries: [
        { key: "settings", ...SETTINGS_NAV },
        ...sections.map(({ id, label, icon }) => ({
          key: `settings-${id}`,
          label: `Settings: ${label}`,
          searchValue: `Settings ${label}`,
          to: `/settings/${id}`,
          icon,
        })),
      ],
    },
  ];
}
