import type { CoreSettings } from "@/api/settings";

/** The stored theme choice, including "system". */
export type ThemeSetting = CoreSettings["theme"];
/** The stored motion choice, including "system". */
export type MotionSetting = CoreSettings["motion"];
/** The palette actually shown. */
export type ResolvedTheme = "light" | "dark";
/** Motion's `reducedMotion`: follow the OS, always reduce, or never reduce. */
export type ReducedMotion = "user" | "always" | "never";

/** Turns the stored theme into a palette, using the OS preference for "system". */
export function resolveTheme(setting: ThemeSetting, systemPrefersDark: boolean): ResolvedTheme {
  if (setting === "system") return systemPrefersDark ? "dark" : "light";
  return setting;
}

const MOTION_MODES: Record<MotionSetting, ReducedMotion> = {
  system: "user",
  reduced: "always",
  full: "never",
};

/** Maps the stored motion choice to Motion's `reducedMotion` mode. */
export function motionMode(setting: MotionSetting): ReducedMotion {
  return MOTION_MODES[setting];
}
