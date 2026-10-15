import type { ModuleManifest } from "./module-manifest";

/**
 * The single list of Modules. Adding a Module = a folder under `src/modules/`
 * plus one entry below, e.g. `import { timeModule } from "@/modules/time";`.
 */
export const modules: readonly ModuleManifest[] = [];
