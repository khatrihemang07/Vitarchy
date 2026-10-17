import { formatCount, formatSpan } from "../format";

/** Which way a figure moved against the Previous Period. */
export type ChangeDirection = "up" | "down" | "none";

/** What a change is counted in. */
export type ChangeUnit = "seconds" | "count";

/** Up, down or unchanged. */
export function changeDirection(delta: number): ChangeDirection {
  if (delta > 0) return "up";
  return delta < 0 ? "down" : "none";
}

const ARROWS: Record<ChangeDirection, string> = { up: "↑", down: "↓", none: "" };

/** "↑ 25m", "↓ 1h 5m" or "—": the arrow carries the meaning, so colour is never the only cue. */
export function formatChange(delta: number, unit: ChangeUnit): string {
  const direction = changeDirection(delta);
  if (direction === "none") return "—";
  const amount = unit === "seconds" ? formatSpan(delta) : formatCount(Math.abs(delta));
  return `${ARROWS[direction]} ${amount}`;
}
