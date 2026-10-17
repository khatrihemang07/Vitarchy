import { Badge, Tooltip, TooltipContent, TooltipTrigger } from "@/design";
import { changeDirection, type ChangeUnit, formatChange } from "./formatChange";

interface ChangeBadgeProps {
  /** Current minus Previous Period; null when there is nothing to compare with. */
  delta: number | null;
  unit: ChangeUnit;
}

/**
 * How a figure moved against the Previous Period. More time is not good or
 * bad, so the colour is quiet and the arrow carries the direction.
 */
export function ChangeBadge({ delta, unit }: ChangeBadgeProps) {
  if (delta === null) return null;
  const direction = changeDirection(delta);
  const text = formatChange(delta, unit);
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Badge
          variant="secondary"
          data-direction={direction}
          aria-label={direction === "none" ? "No change from the Previous Period" : `${text.replace("↑", "up").replace("↓", "down")} from the Previous Period`}
          className="tabular-nums"
        >
          {text}
        </Badge>
      </TooltipTrigger>
      <TooltipContent>Compared with the Previous Period</TooltipContent>
    </Tooltip>
  );
}
