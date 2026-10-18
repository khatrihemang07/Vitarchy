import { useUnconfirmedStatements } from "@/api/finance";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/design";
import { FinanceNotice } from "../FinanceNotice";
import { QueryNotices } from "../QueryNotices";
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
          <QueryNotices inline query={unconfirmed} loadingTitle="Loading Statements…" />
        ) : waiting.length === 0 ? (
          <FinanceNotice inline kind="empty" title="Nothing is waiting to be confirmed." />
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
