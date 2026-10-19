import { changeText } from "@/design";
import { formatCount, formatSpan } from "../format";

/** What a change is counted in. */
export type ChangeUnit = "seconds" | "count";

/** "↑ 25m", "↓ 1h 5m" or "—": the arrow carries the meaning, so colour is never the only cue. */
export function formatChange(delta: number, unit: ChangeUnit): string {
  return changeText(delta, (d) => (unit === "seconds" ? formatSpan(d) : formatCount(Math.abs(d))));
}
