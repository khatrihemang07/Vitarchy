import type { ReactNode } from "react";
import { MotionButton } from "../motion/MotionButton";
import { Card } from "../ui/card";
import { cn } from "../lib/utils";
import { IconChip } from "./IconChip";
import { type NoticeKind, type NoticeView, viewFor } from "./noticeViews";

interface NoticeProps<Extra extends string> {
  /** A built-in kind, or one of the Module's own named in `views`. */
  kind: NoticeKind | Extra;
  /** The Module's own kinds, as data (for example `not-connected`). */
  views?: Readonly<Record<Extra, NoticeView>>;
  /** The one-line message; for a failure, what went wrong. */
  title: string;
  /** Extra explanation under the title. */
  children?: ReactNode;
  /** Shows a Try again button that calls this. */
  onRetry?: () => void;
  /** Another action, such as a link or Clear filters. */
  action?: ReactNode;
  /** Sits inside another card: no surface of its own. */
  inline?: boolean;
}

/** Every loading, failure, empty and warning state of the pages, as one Card with an icon chip. */
export function Notice<Extra extends string = never>({ kind, views, title, children, onRetry, action, inline = false }: NoticeProps<Extra>) {
  const view = viewFor(kind, views);
  return (
    <Card
      role={view.role}
      data-kind={kind}
      className={cn("flex-row flex-wrap items-center gap-4 p-5", view.surface, inline && "border-0 bg-transparent p-0 shadow-none")}
    >
      <IconChip icon={view.icon} tone={view.tone} spins={view.spins} small={inline} />
      <div className="flex min-w-48 flex-1 flex-col gap-1 text-sm">
        <p className={cn("font-semibold", !inline && "font-display text-base")}>{title}</p>
        {children ? <div className="text-muted-foreground">{children}</div> : null}
      </div>
      {onRetry || action ? (
        <div className="flex shrink-0 items-center gap-2">
          {action}
          {onRetry ? (
            <MotionButton variant="outline" size={inline ? "sm" : "default"} onClick={onRetry}>
              Try again
            </MotionButton>
          ) : null}
        </div>
      ) : null}
    </Card>
  );
}
