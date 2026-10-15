import type { CoreSettings } from "@/api/settings";

export type ThemeSetting = CoreSettings["theme"];
export type MotionSetting = CoreSettings["motion"];
export type ResolvedTheme = "light" | "dark";
/** Motion's `reducedMotion`: follow the OS, always reduce, or never reduce. */
export type ReducedMotion = "user" | "always" | "never";

export function resolveTheme(setting: ThemeSetting, systemPrefersDark: boolean): ResolvedTheme {
  if (setting === "system") return systemPrefersDark ? "dark" : "light";
  return setting;
}

const MOTION_MODES: Record<MotionSetting, ReducedMotion> = {
  system: "user",
  reduced: "always",
  full: "never",
};

export function motionMode(setting: MotionSetting): ReducedMotion {
  return MOTION_MODES[setting];
}
