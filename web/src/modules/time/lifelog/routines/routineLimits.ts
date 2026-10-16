/**
 * The limits a Routine is saved within. The server enforces them (`analysis/routines/draft.rs`);
 * the web cannot import Rust, so `scripts/test-guards.sh` fails when these differ from the server's
 * or from the database's CHECK on the name.
 */

/** The most Steps a Routine may have. */
export const MAX_STEPS_PER_ROUTINE = 8;
/** The longest Routine name, in characters. */
export const MAX_NAME_CHARS = 80;
/** The longest Step label, in characters. */
export const MAX_LABEL_CHARS = 80;
/** The longest description or project name a Step matches, in characters. */
export const MAX_VALUE_CHARS = 200;

/** Characters as the server counts them (code points), not UTF-16 units. */
export function charCount(text: string): number {
  return Array.from(text).length;
}

/** `text` cut to at most `max` characters, ending in an ellipsis when something was cut. */
export function truncateChars(text: string, max: number): string {
  const chars = Array.from(text);
  return chars.length > max ? `${chars.slice(0, max - 1).join("")}…` : text;
}
