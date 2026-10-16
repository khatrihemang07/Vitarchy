import type { ReactNode } from "react";
import { useEffect } from "react";
import { useSettings } from "@/api/settings";
import { createLogger } from "@/lib/log";
import { MotionProvider } from "@/design";
import { type Appearance, applyAppearance, readCachedAppearance } from "./apply";
import { motionMode } from "./resolve";
import { useSystemPrefersDark } from "./useSystemPrefersDark";

const log = createLogger("appearance");

const FOLLOW_SYSTEM: Appearance = { theme: "system", motion: "system" };

/**
 * Keeps <html> and Motion in step with the Appearance settings. Until the
 * server answers (or if it never does) the browser's cached choice stands.
 */
export function AppearanceController({ children }: { children: ReactNode }) {
  const { data } = useSettings();
  const systemDark = useSystemPrefersDark();
  const cached = data ? undefined : readCachedAppearance();
  const { theme, motion }: Appearance = data?.core ?? cached ?? FOLLOW_SYSTEM;
  const followingByDefault = !data && !cached;

  useEffect(() => {
    if (followingByDefault) log.debug("no cached appearance; following system");
  }, [followingByDefault]);

  useEffect(() => {
    // Value-free on purpose: theme and motion are the person's choices.
    log.debug("applied appearance");
    applyAppearance({ theme, motion }, systemDark);
  }, [theme, motion, systemDark]);

  return <MotionProvider reducedMotion={motionMode(motion)}>{children}</MotionProvider>;
}
