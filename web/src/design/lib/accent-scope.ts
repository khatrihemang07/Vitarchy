import type { CSSProperties } from "react";

/** Inline style that scopes `--module-accent` (and all `module` utilities) to `color`. */
export function accentScope(color: string): CSSProperties {
  return { "--module-accent": color } as CSSProperties;
}
