/** Query key of the all-settings cache; the one place it is written. */
export const settingsKey = ["settings"] as const;

/** Mutation key of the saves in one scope, nested under [`settingsKey`]. */
export function saveMutationKey(scope: string) {
  return [...settingsKey, "save", scope] as const;
}
