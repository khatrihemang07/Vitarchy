import { appearanceSection } from "../settings/core-sections";
import { optionValues } from "../settings/schema";
import { type MotionSetting, resolveTheme, type ThemeSetting } from "./resolve";

export interface Appearance {
  theme: ThemeSetting;
  motion: MotionSetting;
}

/** Read by public/appearance-boot.js before first paint; its test pins the two together. */
export const APPEARANCE_CACHE_KEY = "vitarchy.appearance";

/** The valid values, derived from the core sections. */
export const THEMES = optionValues(appearanceSection, "theme");
export const MOTIONS = optionValues(appearanceSection, "motion");

export function readCachedAppearance(): Appearance | undefined {
  try {
    const raw = localStorage.getItem(APPEARANCE_CACHE_KEY);
    if (!raw) return undefined;
    const parsed = JSON.parse(raw) as Partial<Record<keyof Appearance, unknown>>;
    const { theme, motion } = parsed;
    if (typeof theme === "string" && typeof motion === "string") {
      if (THEMES.includes(theme) && MOTIONS.includes(motion)) {
        return { theme, motion } as Appearance;
      }
    }
  } catch {
    // Unreadable cache: treat as absent.
  }
  return undefined;
}

/** Set on <html> while the theme swaps, so base.css can pause transitions. */
const SWITCHING_ATTRIBUTE = "data-theme-switching";

/**
 * Without this, each element fades between palettes at its own pace and the
 * page flashes a washed-out grey mid-swap. Transitions resume two frames on,
 * once the new palette has painted.
 */
function pauseTransitionsForOneSwap(root: HTMLElement): void {
  root.setAttribute(SWITCHING_ATTRIBUTE, "");
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      root.removeAttribute(SWITCHING_ATTRIBUTE);
    });
  });
}

/**
 * Puts the appearance on <html> (CSS reads `data-theme` / `data-motion`) and
 * caches the raw settings so the next load paints correctly.
 */
export function applyAppearance(appearance: Appearance, systemPrefersDark: boolean): void {
  const theme = resolveTheme(appearance.theme, systemPrefersDark);
  const root = document.documentElement;
  if (root.dataset.theme !== undefined && root.dataset.theme !== theme) {
    pauseTransitionsForOneSwap(root);
  }
  root.dataset.theme = theme;
  root.dataset.motion = appearance.motion;
  root.style.colorScheme = theme;
  try {
    localStorage.setItem(APPEARANCE_CACHE_KEY, JSON.stringify(appearance));
  } catch {
    // Storage blocked: the cache is a convenience only.
  }
}
