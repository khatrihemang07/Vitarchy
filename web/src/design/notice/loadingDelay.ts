import { parseCssDuration } from "../lib/cssDuration";

/**
 * The `--duration-loading-delay` token in milliseconds: how long a query may stay pending
 * before a skeleton shows. Read from the document so the token stays the one source; 0 when
 * the stylesheet is absent (tests), which shows a skeleton at once.
 */
export function readLoadingDelay(): number {
  return parseCssDuration(getComputedStyle(document.documentElement).getPropertyValue("--duration-loading-delay"));
}
