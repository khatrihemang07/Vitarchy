import { useState } from "react";
import { type ReadAgainCounts, type ReadAgainPreview, type StatementSummary, useReadAgain } from "@/api/finance";
import { Badge, ListRow, MotionButton, Notice } from "@/design";
import { formatDate } from "../dates";
import { errorMessage } from "../failureMessage";
import { formatCount } from "../money";
import { formatLabel, statementKindLabel } from "../statementCopy";
import { ReadAgainPanel } from "./ReadAgainPanel";

/** One confirmed Statement with Read again: preview what would change, then apply it. */
export function StatementRow({ statement }: { statement: StatementSummary }) {
  const readAgain = useReadAgain();
  const [preview, setPreview] = useState<ReadAgainPreview | null>(null);
  const [applied, setApplied] = useState<ReadAgainCounts | null>(null);
  const confirmedOn = formatDate((statement.confirmed_at ?? statement.uploaded_at).slice(0, 10));
  return (
    <ListRow
      title={statement.file_name}
      detail={`Confirmed ${confirmedOn} · ${formatCount(statement.row_count)} rows`}
      below={
        <>
          {readAgain.isError ? <Notice kind="failure" inline title={errorMessage(readAgain.error)} /> : null}
          {preview ? (
            <ReadAgainPanel preview={preview} applied={applied} onApplied={setApplied} onClose={() => { setPreview(null); }} />
          ) : null}
        </>
      }
    >
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
    </ListRow>
  );
}
