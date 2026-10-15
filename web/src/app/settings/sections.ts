import type { ModuleManifest } from "../module-manifest";
import { coreSections } from "./core-sections";
import type { SettingsSection } from "./schema";

/** Core sections, then one per Module that declares `settings`, in registry order. */
export function buildSections(modules: readonly ModuleManifest[]): SettingsSection[] {
  const fromModules = modules.flatMap((m) => (m.settings ? [{ id: m.id, scope: m.id, ...m.settings }] : []));
  return [...coreSections, ...fromModules];
}
