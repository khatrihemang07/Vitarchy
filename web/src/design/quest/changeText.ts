/** Which way a figure moved against what it is compared with. */
export type ChangeDirection = "up" | "down" | "none";

/** Up, down or unchanged. */
export function changeDirection(delta: number): ChangeDirection {
  if (delta > 0) return "up";
  return delta < 0 ? "down" : "none";
}

const ARROWS: Record<ChangeDirection, string> = { up: "↑", down: "↓", none: "" };

/**
 * "↑ 25m", "↓ ₹1,200.00" or "—": the arrow carries the meaning, so colour is never the only
 * cue. `formatAmount` gets the signed change and words the size of it in the figure's unit.
 */
export function changeText(delta: number, formatAmount: (delta: number) => string): string {
  const direction = changeDirection(delta);
  return direction === "none" ? "—" : `${ARROWS[direction]} ${formatAmount(delta)}`;
}
