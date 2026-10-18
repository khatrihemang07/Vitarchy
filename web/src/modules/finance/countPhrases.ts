import { formatCount } from "./money";

/** Joins the non-empty parts with commas. */
export function list(parts: (string | null)[]): string {
  return parts.filter((part): part is string => part !== null).join(", ");
}

/** A phrase for a count that is left out when zero. */
export function whenAny(n: number, phrase: string): string | null {
  return n > 0 ? `${formatCount(n)} ${phrase}` : null;
}
