import type { CSSProperties } from "react";

/** Gap between one section's entrance and the next. */
const STEP_MS = 40;

/** Inline style that delays an `animate-fade-up` entrance by its place in the page, so sections arrive in sequence. */
export function stagger(index: number): CSSProperties {
  return { animationDelay: `${String(Math.max(0, index) * STEP_MS)}ms` };
}
