import type { AnyRoute, AnyRootRoute } from "@tanstack/react-router";
import type { ComponentType } from "react";
import type { ModuleSettings } from "./settings/schema";

export { useModuleSettings } from "./settings/useModuleSettings";
export { withDefaults } from "./settings/schema";
export type {
  ModuleSettings,
  SelectField,
  SettingField,
  SettingsSection,
  SettingsValues,
  SettingValue,
  NumberField,
  TextField,
  ToggleField,
  ValuesOf,
} from "./settings/schema";

/** A screen inside a Module that the command bar can jump to. */
export interface ModulePage {
  readonly id: string;
  readonly label: string;
  readonly to: string;
  /** Defaults to the Module's own icon. */
  readonly icon?: ComponentType<{ className?: string }>;
}

/**
 * What a Module's `src/modules/<id>/index.ts` exports. The shell builds the
 * nav and the router from the list of manifests in `modules.ts`.
 */
export interface ModuleManifest {
  /** Matches the backend Module id; routes live under `/{id}`. */
  readonly id: string;
  readonly label: string;
  /** Accent colour (any CSS colour) the shell may use for this Module. */
  readonly accent: string;
  readonly icon: ComponentType<{ className?: string }>;
  /** Code-based routes, attached under the root route. */
  readonly routes: (root: AnyRootRoute) => AnyRoute[];
  /** Where the nav entry links to. */
  readonly navItem: { readonly to: string };
  /** The screens inside the Module, for the command bar ("Time: Reports"). */
  readonly pages?: readonly ModulePage[];
  /**
   * Optional settings. The Settings page renders them as a section named
   * after the Module id; values are stored under that id.
   */
  readonly settings?: ModuleSettings;
}
