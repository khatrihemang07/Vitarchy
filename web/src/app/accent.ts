/** Accent for Home and any route outside a Module: the brand colour token. */
export const HOME_ACCENT = "var(--brand)";

interface AccentSource {
  readonly id: string;
  readonly accent: string;
}

export interface ResolvedAccent {
  /** The Module whose routes contain the path, or null outside any Module. */
  moduleId: string | null;
  accent: string;
}

/** The Module owning `pathname` (routes live under `/{id}`), if any. */
export function activeModule<T extends { readonly id: string }>(
  pathname: string,
  modules: readonly T[],
): T | undefined {
  return modules.find((m) => pathname === `/${m.id}` || pathname.startsWith(`/${m.id}/`));
}

/** Pure mapping from the current path to the accent the shell should expose. */
export function resolveAccent(pathname: string, modules: readonly AccentSource[]): ResolvedAccent {
  const match = activeModule(pathname, modules);
  return match
    ? { moduleId: match.id, accent: match.accent }
    : { moduleId: null, accent: HOME_ACCENT };
}
