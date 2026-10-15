import { MotionConfig } from "motion/react";
import type { ReactNode } from "react";

interface MotionProviderProps {
  /** "user" follows the OS reduce-motion setting; "always" / "never" override it. */
  reducedMotion?: "user" | "always" | "never";
  children: ReactNode;
}

/** Sets the reduced-motion policy for every Motion animation. */
export function MotionProvider({ reducedMotion = "user", children }: MotionProviderProps) {
  return <MotionConfig reducedMotion={reducedMotion}>{children}</MotionConfig>;
}
