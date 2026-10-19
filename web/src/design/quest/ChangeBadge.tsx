import { Badge } from "../ui/badge";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";
import { changeDirection } from "./changeText";

interface ChangeBadgeProps {
  /** Current minus the reference; null when there is nothing to compare with. */
  delta: number | null;
  /** The badge's words, e.g. "↑ 25m" (see `changeText`). */
  text: string;
  /** What the figure was compared with, e.g. "the Previous Period". */
  against: string;
}

/**
 * How a figure moved against what it is compared with. More or less is not good or bad here,
 * so the colour is quiet and the arrow carries the direction.
 */
export function ChangeBadge({ delta, text, against }: ChangeBadgeProps) {
  if (delta === null) return null;
  const direction = changeDirection(delta);
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Badge
          variant="secondary"
          data-direction={direction}
          aria-label={direction === "none" ? `No change from ${against}` : `${text.replace("↑", "up").replace("↓", "down")} from ${against}`}
          className="tabular-nums"
        >
          {text}
        </Badge>
      </TooltipTrigger>
      <TooltipContent>Compared with {against}</TooltipContent>
    </Tooltip>
  );
}
