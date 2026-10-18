import { useState } from "react";
import { type ConfirmCounts, type StatementPreview, useAccounts, useReadAgain } from "@/api/finance";
import { Card, CardContent } from "@/design";
import { confirmedLine } from "./importCopy";
import { unreadCopy } from "../statementCopy";
import { NotReadNotice } from "./NotReadNotice";
import { ReviewBody } from "./ReviewBody";
import { StatementActions } from "./StatementActions";
import { StatementHeader } from "./StatementHeader";

/** One Statement's preview: what was read, where it goes, and Confirm or Discard. */
export function StatementCard({ preview: first }: { preview: StatementPreview }) {
  const [preview, setPreview] = useState(first);
  const [tries, setTries] = useState(0);
  const [confirmed, setConfirmed] = useState<ConfirmCounts | null>(null);
  const [discarded, setDiscarded] = useState(false);
  const accounts = useAccounts();
  const readAgain = useReadAgain();
  const unread = unreadCopy(preview);

  if (discarded) return null;
  return (
    <Card data-statement-id={preview.id} aria-label={preview.file_name}>
      <StatementHeader preview={preview} />
      {confirmed ? (
        <CardContent className="text-sm" role="status">
          <span className="font-medium">Confirmed.</span>{" "}
          <span className="text-muted-foreground">
            {confirmedLine(confirmed)}
          </span>
        </CardContent>
      ) : unread ? (
        <CardContent className="flex flex-col gap-4">
          <NotReadNotice
            copy={unread}
            outcome={preview.outcome}
            retrying={readAgain.isPending}
            retryError={readAgain.error}
            onTryAgain={() => {
              readAgain.mutate(preview.id, {
                onSuccess: (result) => {
                  setPreview(result.statement);
                  setTries((n) => n + 1);
                },
              });
            }}
          />
          <StatementActions statementId={preview.id} onDiscarded={() => { setDiscarded(true); }} />
        </CardContent>
      ) : (
        <ReviewBody
          key={tries}
          preview={preview}
          accounts={accounts.data ?? []}
          onConfirmed={setConfirmed}
          onDiscarded={() => { setDiscarded(true); }}
        />
      )}
    </Card>
  );
}
