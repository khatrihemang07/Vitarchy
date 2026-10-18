import { CircleCheck, CircleSlash, TriangleAlert } from "lucide-react";
import type { ComponentType } from "react";
import type { CheckView } from "@/api/finance";
import { Badge, cn } from "@/design";
import { checkCopy, hasFailedCheck } from "../statementCopy";

interface OutcomeView {
  label: string;
  icon: ComponentType<{ className?: string }>;
  tone: string;
}

/** How each outcome of a Check looks: a failed Check is a warning, never an error. */
const OUTCOME_VIEWS: Record<string, OutcomeView> = {
  passed: { label: "Passed", icon: CircleCheck, tone: "bg-success/15 text-success" },
  failed: { label: "Failed", icon: TriangleAlert, tone: "bg-warning/15 text-warning" },
  skipped: { label: "Skipped", icon: CircleSlash, tone: "bg-muted text-muted-foreground" },
};

/** A section's Checks as badges with plain-language text; a failed one warns without stopping a confirm. */
export function ChecksList({ checks }: { checks: readonly CheckView[] }) {
  if (checks.length === 0) return null;
  return (
    <div className="flex flex-col gap-2">
      <ul className="flex flex-col gap-2 text-sm">
        {checks.map((check) => {
          const view = OUTCOME_VIEWS[check.outcome] ?? OUTCOME_VIEWS.skipped;
          const copy = checkCopy(check);
          const Icon = view?.icon ?? CircleSlash;
          return (
            <li key={check.name} data-outcome={check.outcome} className="flex flex-wrap items-center gap-x-3 gap-y-1">
              <Badge variant="ghost" className={cn("border-0", view?.tone)}>
                <Icon aria-hidden="true" />
                {view?.label ?? check.outcome}
              </Badge>
              <span className="font-medium">{copy.name}</span>
              <span className="min-w-48 flex-1 text-muted-foreground">{copy.text}</span>
            </li>
          );
        })}
      </ul>
      {hasFailedCheck(checks) ? (
        <p className="text-sm text-warning">A failed Check is a warning: you can still confirm this Statement.</p>
      ) : null}
    </div>
  );
}
