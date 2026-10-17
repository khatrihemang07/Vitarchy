import { Badge, Tooltip, TooltipContent, TooltipTrigger } from "@/design";
import { COMPARE, type CompareMode } from "./compareMode";
import { changeDirection, type ChangeUnit, formatChange } from "./formatChange";

interface ChangeBadgeProps {
  /** Current minus the reference; null when there is nothing to compare with. */
  delta: number | null;
  unit: ChangeUnit;
  /** What the figure was compared with. */
  against: CompareMode;
}

/**
 * How a figure moved against the Previous Period or the Usual. More time is not good or
 * bad, so the colour is quiet and the arrow carries the direction.
 */
export function ChangeBadge({ delta, unit, against }: ChangeBadgeProps) {
  if (delta === null) return null;
  const direction = changeDirection(delta);
  const text = formatChange(delta, unit);
  const phrase = COMPARE[against].phrase;
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Badge
          variant="secondary"
          data-direction={direction}
          aria-label={direction === "none" ? `No change from ${phrase}` : `${text.replace("↑", "up").replace("↓", "down")} from ${phrase}`}
          className="tabular-nums"
        >
          {text}
        </Badge>
      </TooltipTrigger>
      <TooltipContent>Compared with {phrase}</TooltipContent>
    </Tooltip>
  );
}
