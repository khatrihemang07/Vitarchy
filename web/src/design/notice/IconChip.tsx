import type { ComponentType } from "react";
import { cn } from "../lib/utils";

interface IconChipProps {
  icon: ComponentType<{ className?: string }>;
  /** Classes tinting the chip, such as `bg-warning/15 text-warning`. */
  tone: string;
  /** Whether the icon turns, for work in progress. */
  spins?: boolean;
  /** A smaller chip for notices that sit inside another card. */
  small?: boolean;
}

/** A round tinted chip around an icon: the mark at the start of every notice and status card. */
export function IconChip({ icon: Icon, tone, spins = false, small = false }: IconChipProps) {
  return (
    <span className={cn("grid shrink-0 place-items-center rounded-full", tone, small ? "size-9" : "size-11")}>
      <Icon className={cn("size-5", spins && "animate-spin")} aria-hidden="true" />
    </span>
  );
}
