/** A finite number within the field's bounds, or undefined if the text is not one. */
export function parseNumber(text: string, min?: number, max?: number): number | undefined {
  if (text.trim() === "") return undefined;
  const n = Number(text);
  if (!Number.isFinite(n)) return undefined;
  return Math.min(max ?? Infinity, Math.max(min ?? -Infinity, n));
}
