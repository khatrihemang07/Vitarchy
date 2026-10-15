import { Home } from "lucide-react";
import type { ComponentType } from "react";
import type { ModuleManifest } from "./module-manifest";

export interface NavEntry {
  id: string;
  label: string;
  to: string;
  accent: string;
  icon: ComponentType<{ className?: string }>;
}

/** Home is not a Module, so it has no manifest or accent; the shell and command bar both list it from here. */
export const HOME_NAV = { label: "Home", to: "/", icon: Home } as const;

export function buildNav(modules: readonly ModuleManifest[]): NavEntry[] {
  return modules.map((m) => ({
    id: m.id,
    label: m.label,
    to: m.navItem.to,
    accent: m.accent,
    icon: m.icon,
  }));
}
