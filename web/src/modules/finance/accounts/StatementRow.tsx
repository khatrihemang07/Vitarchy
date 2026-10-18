import { useState } from "react";
import { type ReadAgainCounts, type ReadAgainPreview, type StatementSummary, useReadAgain } from "@/api/finance";
import { Badge, MotionButton } from "@/design";
import { formatDate } from "../dates";
import { errorMessage } from "../failureMessage";
import { FinanceNotice } from "../FinanceNotice";
import { formatCount } from "../money";
import { formatLabel, statementKindLabel } from "../statementCopy";
import { ReadAgainPanel } from "./ReadAgainPanel";

/** One confirmed Statement with Read again: preview what would change, then apply it. */
export function StatementRow({ statement }: { statement: StatementSummary }) {
  const readAgain = useReadAgain();
  const [preview, setPreview] = useState<ReadAgainPreview | null>(null);
  const [applied, setApplied] = useState<ReadAgainCounts | null>(null);
  return (
    <li className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-xl border bg-card p-4">
        <div className="flex min-w-48 flex-1 flex-col gap-1">
          <p className="font-medium break-all">{statement.file_name}</p>
          <p className="text-sm text-muted-foreground">
            Confirmed {formatDate((statement.confirmed_at ?? statement.uploaded_at).slice(0, 10))} · {formatCount(statement.row_count)} rows
          </p>
        </div>
        <Badge variant="outline">{formatLabel(statement.format)}</Badge>
        <Badge variant="secondary">{statementKindLabel(statement.kind)}</Badge>
        <MotionButton
          variant="outline"
          size="sm"
          disabled={readAgain.isPending}
          onClick={() => {
            setApplied(null);
            readAgain.mutate(statement.id, { onSuccess: setPreview });
          }}
        >
          {readAgain.isPending ? "Reading…" : "Read again"}
        </MotionButton>
      </div>
      {readAgain.isError ? <FinanceNotice kind="failure" inline title={errorMessage(readAgain.error)} /> : null}
      {preview ? (
        <ReadAgainPanel preview={preview} applied={applied} onApplied={setApplied} onClose={() => { setPreview(null); }} />
      ) : null}
    </li>
  );
}
