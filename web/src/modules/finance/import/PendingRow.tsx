import type { StatementSummary } from "@/api/finance";
import { Badge, ListRow, MotionButton } from "@/design";
import { formatDate } from "../dates";
import { formatCount } from "../money";
import { formatLabel, statementKindLabel } from "../statementCopy";

/** One Statement from an earlier session that still waits to be confirmed; Review moves it up to the cards being reviewed. */
export function PendingRow({ statement, onReview }: { statement: StatementSummary; onReview: (id: number) => void }) {
  return (
    <ListRow title={statement.file_name} detail={`Uploaded ${formatDate(statement.uploaded_at)} · ${formatCount(statement.row_count)} rows`}>
      <Badge variant="outline">{formatLabel(statement.format)}</Badge>
      <Badge variant="secondary">{statementKindLabel(statement.kind)}</Badge>
      {statement.outcome === "read" ? null : <Badge variant="outline">Not read</Badge>}
      <MotionButton variant="outline" size="sm" onClick={() => { onReview(statement.id); }}>
        Review
      </MotionButton>
    </ListRow>
  );
}
