import { Home, Settings } from "lucide-react";
import type { ComponentType } from "react";
import type { ModuleManifest, ModulePage } from "./module-manifest";
import { generalSection } from "./settings/core-sections";

/** One sidebar and command-bar entry for a Module. */
export interface NavEntry {
  id: string;
  label: string;
  to: string;
  accent: string;
  icon: ComponentType<{ className?: string }>;
  /** The screens inside the Module; empty when it has none. */
  pages: readonly ModulePage[];
}

/** Home is not a Module, so it has no manifest or accent; the shell and command bar both list it from here. */
export const HOME_NAV = { label: "Home", to: "/", icon: Home } as const;

/** Settings is Core too: the gear in the sidebar and the command bar both link here. */
export const SETTINGS_NAV = { label: "Settings", to: `/settings/${generalSection.id}`, icon: Settings } as const;

/** Nav entries for the registered Modules, in registry order. */
export function buildNav(modules: readonly ModuleManifest[]): NavEntry[] {
  return modules.map((m) => ({
    id: m.id,
    label: m.label,
    to: m.navItem.to,
    accent: m.accent,
    icon: m.icon,
    pages: m.pages ?? [],
  }));
}
