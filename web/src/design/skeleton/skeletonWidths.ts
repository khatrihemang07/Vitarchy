/** Widths cycled through the cells of a skeleton row, so rows look read rather than ruled. */
const WIDTHS = ["w-2/3", "w-1/2", "w-5/6", "w-3/5", "w-3/4"] as const;

/** The width class for a skeleton cell, varying with its row and column but identical on every render. */
export function skeletonWidth(row: number, column: number): (typeof WIDTHS)[number] {
  return WIDTHS[(row * 2 + column) % WIDTHS.length] ?? WIDTHS[0];
}
