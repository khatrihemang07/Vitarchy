import { useUnconfirmedStatements } from "@/api/finance";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, Notice, QueryNotices, TableSkeleton } from "@/design";
import { errorMessage } from "../failureMessage";
import { PendingRow } from "./PendingRow";

/** Statements from earlier sessions that still wait to be confirmed (the ones just uploaded show above). */
export function PendingStatements({ excludeIds, onReview }: { excludeIds: ReadonlySet<number>; onReview: (id: number) => void }) {
  const unconfirmed = useUnconfirmedStatements();
  const waiting = unconfirmed.data?.filter((s) => !excludeIds.has(s.id));
  return (
    <Card>
      <CardHeader>
        <CardTitle>Waiting to be confirmed</CardTitle>
        <CardDescription>Statements you uploaded earlier and have not confirmed or discarded.</CardDescription>
      </CardHeader>
      <CardContent>
        {!waiting ? (
          <QueryNotices inline query={unconfirmed} describeError={errorMessage} skeleton={<TableSkeleton label="Loading Statements" rows={2} columns={3} />} />
        ) : waiting.length === 0 ? (
          <Notice inline kind="empty" title="Nothing is waiting to be confirmed." />
        ) : (
          <ul className="flex flex-col gap-3">
            {waiting.map((statement) => (
              <PendingRow key={statement.id} statement={statement} onReview={onReview} />
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
