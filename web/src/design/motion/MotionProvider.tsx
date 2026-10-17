import { domMax, LazyMotion, MotionConfig } from "motion/react";
import type { ReactNode } from "react";

interface MotionProviderProps {
  /** "user" follows the OS reduce-motion setting; "always" / "never" override it. */
  reducedMotion?: "user" | "always" | "never";
  children: ReactNode;
}

/**
 * Sets the reduced-motion policy for every Motion animation and supplies the animation
 * features. Animated elements are `m.*`, never `motion.*` (which carries its own copy of
 * every feature). domMax rather than domAnimation because the nav indicator uses `layoutId`.
 */
export function MotionProvider({ reducedMotion = "user", children }: MotionProviderProps) {
  return (
    <LazyMotion features={domMax} strict>
      <MotionConfig reducedMotion={reducedMotion}>{children}</MotionConfig>
    </LazyMotion>
  );
}
