import { Flame } from "lucide-react";
import { cn } from "../lib/utils";
import { streakLabel, streakTier, type StreakTier } from "./streak";

const flameClass: Record<StreakTier, string> = {
  cold: "text-muted-foreground",
  warm: "fill-flame text-flame",
  hot: "animate-flame fill-flame-hot text-flame-hot drop-shadow-flame-hot",
};

interface StreakFlameProps {
  days: number;
  className?: string;
}

export function StreakFlame({ days, className }: StreakFlameProps) {
  const tier = streakTier(days);
  return (
    <span className={cn("inline-flex items-center gap-2 text-sm font-medium", className)}>
      <Flame
        data-testid="streak-flame"
        data-tier={tier}
        aria-hidden="true"
        className={cn("size-6 origin-bottom", flameClass[tier])}
      />
      <span className={tier === "cold" ? "text-muted-foreground" : undefined}>{streakLabel(days)}</span>
    </span>
  );
}
