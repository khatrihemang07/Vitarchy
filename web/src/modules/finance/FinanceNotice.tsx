import type { ReactNode } from "react";
import { Card, cn, MotionButton } from "@/design";
import { NOTICE_VIEWS, type NoticeKind } from "./noticeKinds";

interface FinanceNoticeProps {
  kind: NoticeKind;
  /** The one-line message; for a failure, what went wrong. */
  title: string;
  /** Extra explanation under the title. */
  children?: ReactNode;
  /** Shows a Try again button that calls this. */
  onRetry?: () => void;
  /** Another action, such as a link. */
  action?: ReactNode;
  /** Sits inside another card: no surface of its own. */
  inline?: boolean;
}

/** Every loading, failure, empty and warning state of the Finance pages, as one Card with an icon chip. */
export function FinanceNotice({ kind, title, children, onRetry, action, inline = false }: FinanceNoticeProps) {
  const view = NOTICE_VIEWS[kind];
  const Icon = view.icon;
  return (
    <Card
      role={view.role}
      data-kind={kind}
      className={cn("flex-row flex-wrap items-center gap-4 p-5", view.surface, inline && "border-0 bg-transparent p-0 shadow-none")}
    >
      <span className={cn("grid shrink-0 place-items-center rounded-full", view.tone, inline ? "size-9" : "size-11")}>
        <Icon className={cn("size-5", view.spins && "animate-spin")} aria-hidden="true" />
      </span>
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
