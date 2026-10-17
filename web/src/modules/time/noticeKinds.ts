import { AlertTriangle, Inbox, Info, Loader2, PlugZap } from "lucide-react";
import type { ComponentType } from "react";

/** The situations a Time page explains instead of showing data. */
export type NoticeKind = "loading" | "failure" | "empty" | "not-connected" | "settings-fallback";

interface NoticeView {
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

/** How each kind looks, as data, mirroring the views of the Home health card. */
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
  "not-connected": { icon: PlugZap, tone: "bg-module/15 text-module", surface: "", role: "status", spins: false },
  "settings-fallback": {
    icon: Info,
    tone: "bg-warning/15 text-warning",
    surface: "border-warning/50 bg-warning/10",
    role: "status",
    spins: false,
  },
} as const satisfies Record<NoticeKind, NoticeView>;
