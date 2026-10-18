import type { StatementSummary } from "@/api/finance";
import { Badge, MotionButton } from "@/design";
import { formatDate } from "../dates";
import { formatCount } from "../money";
import { formatLabel, statementKindLabel } from "../statementCopy";

/** One Statement from an earlier session that still waits to be confirmed; Review moves it up to the cards being reviewed. */
export function PendingRow({ statement, onReview }: { statement: StatementSummary; onReview: (id: number) => void }) {
  return (
    <li>
      <div className="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-xl border bg-card p-4">
        <div className="flex min-w-48 flex-1 flex-col gap-1">
          <p className="font-medium break-all">{statement.file_name}</p>
          <p className="text-sm text-muted-foreground">
            Uploaded {formatDate(statement.uploaded_at)} · {formatCount(statement.row_count)} rows
          </p>
        </div>
        <Badge variant="outline">{formatLabel(statement.format)}</Badge>
        <Badge variant="secondary">{statementKindLabel(statement.kind)}</Badge>
        {statement.outcome === "read" ? null : <Badge variant="outline">Not read</Badge>}
        <MotionButton variant="outline" size="sm" onClick={() => { onReview(statement.id); }}>
          Review
        </MotionButton>
      </div>
    </li>
  );
}
