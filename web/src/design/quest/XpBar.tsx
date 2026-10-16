import { motion } from "motion/react";
import { cn } from "../lib/utils";
import { springs } from "../motion/springs";
import { xpProgress } from "./xp";

interface XpBarProps {
  value: number;
  max: number;
  /** Accessible name, e.g. "Level 12 progress". */
  label: string;
  className?: string;
}

/** Progress toward the next level. */
export function XpBar({ value, max, label, className }: XpBarProps) {
  const progress = xpProgress(value, max);
  return (
    <div
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={progress.max}
      aria-valuenow={progress.value}
      aria-valuetext={`${String(progress.value)} of ${String(progress.max)} XP`}
      className={cn(
        "h-3 overflow-hidden rounded-full border-2 border-border bg-xp-track",
        className,
      )}
    >
      <motion.div
        className="h-full rounded-full bg-linear-to-r from-xp-from to-xp-to"
        initial={{ width: "0%" }}
        animate={{ width: `${String(progress.percent)}%` }}
        transition={springs.gentle}
      />
    </div>
  );
}
