import type { ComponentType, ReactNode } from "react";
import { cn } from "../lib/utils";
import { Pressable } from "../motion/Pressable";
import { Card } from "../ui/card";

interface StatCardProps {
  label: string;
  /** Usually a number or a NumberTicker. */
  value: ReactNode;
  hint?: string;
  icon?: ComponentType<{ className?: string }>;
  className?: string;
}

/** A stat tile tinted by the active module accent. */
export function StatCard({ label, value, hint, icon: Icon, className }: StatCardProps) {
  return (
    <Pressable>
      <Card className={cn("surface-chroma gap-2 p-5", className)}>
        <div className="flex items-center justify-between text-sm text-muted-foreground">
          <span>{label}</span>
          {Icon ? <Icon className="size-4 text-module" /> : null}
        </div>
        <div className="font-display text-3xl font-bold tracking-tight">{value}</div>
        {hint ? <p className="text-xs text-muted-foreground">{hint}</p> : null}
      </Card>
    </Pressable>
  );
}
