/**
 * The whitespace Rust's `str::trim` removes (Unicode White_Space). JS `trim`
 * differs at the edges (it also strips U+FEFF but not U+0085), and the client
 * must agree with the server about what a trimmed name is.
 */
const TRIMMED = "\\t\\n\\v\\f\\r \\u0085\\u00a0\\u1680\\u2000-\\u200a\\u2028\\u2029\\u202f\\u205f\\u3000";
const EDGE_WHITESPACE = new RegExp(`^[${TRIMMED}]+|[${TRIMMED}]+$`, "g");

/** Length in Unicode code points, as Rust's `chars().count()` (not UTF-16 units). */
export function codePointLength(text: string): number {
  return Array.from(text).length;
}

/**
 * What the server will store for `text`: trimmed, then cut to `maxLength`
 * code points (never inside an emoji). No limit when `maxLength` is omitted.
 */
export function normalizeText(text: string, maxLength?: number): string {
  const trimmed = text.replace(EDGE_WHITESPACE, "");
  if (maxLength === undefined) return trimmed;
  return Array.from(trimmed).slice(0, maxLength).join("");
}
