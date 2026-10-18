import { KeyRound } from "lucide-react";
import type { StatementPreview } from "@/api/finance";
import { Badge, CardDescription, CardHeader, CardTitle } from "@/design";
import { formatPeriod } from "../dates";
import { formatLabel, statementKindLabel } from "../statementCopy";

/** The file's name, format, kind and period, and the Statement password that opened it. */
export function StatementHeader({ preview }: { preview: StatementPreview }) {
  return (
    <CardHeader>
      <CardTitle className="break-all">{preview.file_name}</CardTitle>
      <CardDescription className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
        <Badge variant="outline">{formatLabel(preview.format)}</Badge>
        <Badge variant="secondary">{statementKindLabel(preview.kind)}</Badge>
        {preview.outcome === "read" && preview.kind !== "payment_history" ? <span>{formatPeriod(preview.period)}</span> : null}
        {preview.unlocked_with_label ? (
          <span className="flex items-center gap-1">
            <KeyRound aria-hidden="true" className="size-3.5" />
            Opened with {preview.unlocked_with_label}
          </span>
        ) : null}
      </CardDescription>
    </CardHeader>
  );
}
