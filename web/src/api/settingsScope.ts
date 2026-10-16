import type { components } from "./schema";

/** The Core settings, typed from the server's OpenAPI document. */
export type CoreSettings = components["schemas"]["CoreSettings"];
/** A Module's settings: its own JSON object. */
export type ModuleSettingsValues = Record<string, unknown>;
/** Every scope's settings as the server returns them. */
export interface AllSettings {
  core: CoreSettings;
  /** Keyed by Module id. */
  modules: Record<string, ModuleSettingsValues>;
}

/** The scope under which the core settings are stored; Modules use their id. */
export const CORE_SCOPE = "core";

/** The stored object for a scope, or {} if nothing is saved there yet. */
export function storedFor(all: AllSettings, scope: string): Record<string, unknown> {
  return scope === CORE_SCOPE ? { ...all.core } : { ...all.modules[scope] };
}

/** `all` with one scope's settings replaced by `values`. */
export function withScope(all: AllSettings, scope: string, values: Record<string, unknown>): AllSettings {
  return scope === CORE_SCOPE
    ? { ...all, core: values as CoreSettings }
    : { ...all, modules: { ...all.modules, [scope]: values } };
}

