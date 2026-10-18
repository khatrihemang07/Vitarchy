import type { CSSProperties } from "react";
import type { Away } from "@/api/time";

/** How much of the track shows through a mark that is not highlighted. */
const DIMMED_OPACITY = 0.25;

/** The same hatch the Away legend swatches use, from tokens. */
export const AWAY_HATCH = "repeating-linear-gradient(45deg, var(--away) 0 2px, transparent 2px 6px)";

/**
 * How a mark looks: the app colour when Active, hatched when Away, and the app colour
 * washed toward the unknown token when Away is unknown. Texture, not just colour, tells them apart.
 */
export function markStyle(away: Away, color: string, dimmed: boolean): CSSProperties {
  const opacity = dimmed ? DIMMED_OPACITY : 1;
  if (away === "yes") return { backgroundImage: AWAY_HATCH, boxShadow: "inset 0 0 0 1px var(--away)", opacity };
  if (away === "unknown") return { background: `color-mix(in srgb, ${color} 40%, var(--away-unknown))`, opacity };
  return { background: color, opacity };
}
