import { financeModule } from "@/modules/finance";
import { timeModule } from "@/modules/time";
import type { ModuleManifest } from "./module-manifest";

/**
 * The single list of Modules. Adding a Module = a folder under `src/modules/`
 * plus one entry below.
 */
export const modules: readonly ModuleManifest[] = [timeModule, financeModule];
