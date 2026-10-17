import { MotionConfigContext, useReducedMotion } from "motion/react";
import { useContext } from "react";

/** Whether animation should be skipped: the Motion setting forces it, or the OS asks for it and the setting leaves it to the OS. */
export function useMotionReduced(): boolean {
  const { reducedMotion } = useContext(MotionConfigContext);
  const osReduced = useReducedMotion() ?? false;
  if (reducedMotion === "always") return true;
  if (reducedMotion === "never") return false;
  return osReduced;
}
