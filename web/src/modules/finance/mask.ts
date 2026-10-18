/** A masked number as one consistent display: trimmed and upper-cased ("xxxxxxx1111" becomes "XXXXXXX1111"); empty when none was printed. */
export function formatMask(mask: string | null | undefined): string {
  return (mask ?? "").trim().toUpperCase();
}

/** The masked number for a card, or words saying none was printed. */
export function maskOrNone(mask: string | null | undefined): string {
  return formatMask(mask) || "No number printed";
}
