import type { ReactNode } from "react";
import { useEffect } from "react";
import { useSettings } from "@/api/settings";
import { MotionProvider } from "@/design";
import { type Appearance, applyAppearance, readCachedAppearance } from "./apply";
import { motionMode } from "./resolve";
import { useSystemPrefersDark } from "./useSystemPrefersDark";

const FOLLOW_SYSTEM: Appearance = { theme: "system", motion: "system" };

/**
 * Keeps <html> and Motion in step with the Appearance settings. Until the
 * server answers (or if it never does) the browser's cached choice stands.
 */
export function AppearanceController({ children }: { children: ReactNode }) {
  const { data } = useSettings();
  const systemDark = useSystemPrefersDark();
  const { theme, motion }: Appearance = data?.core ?? readCachedAppearance() ?? FOLLOW_SYSTEM;

  useEffect(() => {
    applyAppearance({ theme, motion }, systemDark);
  }, [theme, motion, systemDark]);

  return <MotionProvider reducedMotion={motionMode(motion)}>{children}</MotionProvider>;
}
