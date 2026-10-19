import { cn } from "../lib/utils";

interface ProgressProps {
  /** Accessible name: what is in progress. */
  label: string;
  /** 0 to 100; leave out while the length of the work is unknown. */
  value?: number;
  className?: string;
}

/** A thin progress bar; with no `value` it pulses to show work is under way. */
export function Progress({ label, value, className }: ProgressProps) {
  const known = value !== undefined;
  return (
    <div
      role="progressbar"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={known ? Math.round(value) : undefined}
      className={cn("h-1.5 overflow-hidden rounded-full bg-muted", className)}
    >
      <div
        className={cn("h-full rounded-full bg-module transition-[width]", !known && "w-full motion-safe:animate-pulse")}
        style={known ? { width: `${String(Math.min(100, Math.max(0, value)))}%` } : undefined}
      />
    </div>
  );
}
