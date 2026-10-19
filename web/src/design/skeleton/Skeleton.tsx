import type { ComponentProps } from "react";
import { cn } from "../lib/utils";

/** A pulsing placeholder block; size it with classes. Purely visual, so hidden from assistive tech. */
export function Skeleton({ className, ...props }: ComponentProps<"div">) {
  return <div aria-hidden="true" data-slot="skeleton" className={cn("animate-pulse rounded-md bg-muted", className)} {...props} />;
}
