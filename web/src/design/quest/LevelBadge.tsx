import { cn } from "../lib/utils";
import { normalizeLevel } from "./level";

interface LevelBadgeProps {
  level: number;
  className?: string;
}

export function LevelBadge({ level, className }: LevelBadgeProps) {
  const shown = normalizeLevel(level);
  return (
    <span
      aria-label={`Level ${String(shown)}`}
      className={cn(
        "inline-flex size-11 shrink-0 flex-col items-center justify-center rounded-full",
        "bg-linear-to-br from-xp-from to-xp-to font-display font-bold text-on-accent",
        "shadow-key leading-none",
        className,
      )}
    >
      <span aria-hidden="true" className="text-2xs tracking-widest uppercase opacity-80">
        Lv
      </span>
      <span aria-hidden="true" className="text-base">
        {shown}
      </span>
    </span>
  );
}
