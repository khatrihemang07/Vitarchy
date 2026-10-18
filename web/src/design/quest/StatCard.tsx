import type { ComponentType, ReactNode } from "react";
import { cn } from "../lib/utils";
import { Pressable } from "../motion/Pressable";
import { Card } from "../ui/card";

interface StatCardProps {
  label: string;
  /** Usually a number or a NumberTicker. */
  value: ReactNode;
  hint?: string;
  /** Sits beside the value (e.g. a change from the last Period). */
  badge?: ReactNode;
  icon?: ComponentType<{ className?: string }>;
  className?: string;
}

/** A stat tile tinted by the active module accent. */
export function StatCard({ label, value, hint, badge, icon: Icon, className }: StatCardProps) {
  return (
    <Pressable className="h-full">
      <Card className={cn("surface-chroma h-full gap-2 p-5", className)}>
        <div className="flex items-center justify-between text-sm text-muted-foreground">
          <span>{label}</span>
          {Icon ? <Icon className="size-4 text-module" /> : null}
        </div>
        <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
          <span className="font-display text-3xl font-bold tracking-tight">{value}</span>
          {badge}
        </div>
        {hint ? <p className="text-xs text-muted-foreground">{hint}</p> : null}
      </Card>
    </Pressable>
  );
}
