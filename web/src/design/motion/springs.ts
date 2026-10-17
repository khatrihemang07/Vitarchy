import type { Transition } from "motion/react";

/** Spring presets, defined once and reused by every animated primitive. */
export const springs = {
  /** Hover/press on buttons, nav items and cards: tactile, with overshoot. */
  bouncy: { type: "spring", stiffness: 500, damping: 18 },
  /** Layout moves such as the nav indicator. */
  smooth: { type: "spring", stiffness: 380, damping: 30 },
  /** Bars and gauges filling in. */
  gentle: { type: "spring", stiffness: 120, damping: 20 },
} as const satisfies Record<string, Transition>;

/** The one hover/press interaction; spread onto any `m.*` element. */
export const pressable = {
  whileHover: { scale: 1.04 },
  whileTap: { scale: 0.95 },
  transition: springs.bouncy,
} as const;

/** Rows that slide toward the pointer on hover. */
export const nudge = {
  whileHover: { x: 4 },
  transition: springs.bouncy,
} as const;
