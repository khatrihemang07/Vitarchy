import { Activity, DatabaseZap, ServerOff } from "lucide-react";
import type { ComponentType } from "react";
import { useHealth, type HealthState } from "@/api/health";
import { Card, cn } from "@/design";

type View = {
  state: "checking" | "ok" | "degraded" | "offline";
  title: string;
  detail: string;
  icon: ComponentType<{ className?: string }>;
  /** Classes tinting the icon chip. */
  tone: string;
  /** Classes tinting the card itself. */
  surface: string;
};

const views = {
  ok: {
    state: "ok",
    title: "Server ok, database ok",
    detail: "Everything is running locally.",
    icon: Activity,
    tone: "bg-success/15 text-success",
    surface: "",
  },
  "db-down": {
    state: "degraded",
    title: "Server up, database down",
    detail: "Start Postgres with docker compose up -d.",
    icon: DatabaseZap,
    tone: "bg-warning/15 text-warning",
    surface: "border-warning/50 bg-warning/10 shadow-glow-warning",
  },
  unreachable: {
    state: "offline",
    title: "Server unreachable",
    detail: "Start it with cargo run -p vitarchy-server.",
    icon: ServerOff,
    tone: "bg-destructive/15 text-destructive",
    surface: "border-destructive/50 bg-destructive/10 shadow-glow-destructive",
  },
} as const satisfies Record<HealthState["kind"], View>;

const checking: View = {
  state: "checking",
  title: "Checking server…",
  detail: "Asking the local server how it is doing.",
  icon: Activity,
  tone: "bg-muted text-muted-foreground",
  surface: "",
};

export function HealthCard() {
  const { data, isPending } = useHealth();
  const view = isPending || !data ? checking : views[data.kind];
  const Icon = view.icon;
  return (
    <Card
      role="status"
      data-state={view.state}
      className={cn("flex-row items-center gap-4 p-5", view.surface)}
    >
      <span className={cn("grid size-11 shrink-0 place-items-center rounded-full", view.tone)}>
        <Icon className="size-5" />
      </span>
      <div className="min-w-0">
        <p className="text-xs tracking-widest text-muted-foreground uppercase">System health</p>
        <p className="font-display font-semibold">{view.title}</p>
        <p className="text-sm text-muted-foreground">{view.detail}</p>
      </div>
    </Card>
  );
}
