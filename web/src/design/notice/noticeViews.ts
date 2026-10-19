import { AlertTriangle, Inbox, Loader2, TriangleAlert } from "lucide-react";
import type { ComponentType } from "react";

/** The situations a page explains instead of showing data. A Module may add its own (see `Notice`). */
export type NoticeKind = "loading" | "failure" | "empty" | "warning";

/** How one kind of notice looks, as data. */
export interface NoticeView {
  icon: ComponentType<{ className?: string }>;
  /** Classes tinting the icon chip. */
  tone: string;
  /** Classes tinting the card itself. */
  surface: string;
  /** Failures interrupt the reader; the rest wait to be read. */
  role: "alert" | "status";
  /** Whether the icon turns, for work in progress. */
  spins: boolean;
}

/** Everything a view needs except its icon: the one look all warning-style kinds share. */
export const WARNING_STYLE = {
  tone: "bg-warning/15 text-warning",
  surface: "border-warning/50 bg-warning/10",
  role: "status",
  spins: false,
} as const satisfies Omit<NoticeView, "icon">;

/** The views of the built-in kinds. */
export const NOTICE_VIEWS = {
  loading: { icon: Loader2, tone: "bg-muted text-muted-foreground", surface: "", role: "status", spins: true },
  failure: {
    icon: AlertTriangle,
    tone: "bg-destructive/15 text-destructive",
    surface: "border-destructive/50 bg-destructive/10 shadow-glow-destructive",
    role: "alert",
    spins: false,
  },
  empty: { icon: Inbox, tone: "bg-muted text-muted-foreground", surface: "", role: "status", spins: false },
  warning: { icon: TriangleAlert, ...WARNING_STYLE },
} as const satisfies Record<NoticeKind, NoticeView>;

/** The view for a kind: the Module's own views win, then the built-in ones. */
export function viewFor<Extra extends string>(kind: NoticeKind | Extra, extra?: Readonly<Record<Extra, NoticeView>>): NoticeView {
  if (extra && kind in extra) return extra[kind as Extra];
  return NOTICE_VIEWS[kind as NoticeKind];
}
