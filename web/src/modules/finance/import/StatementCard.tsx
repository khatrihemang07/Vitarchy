import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { type ConfirmCounts, type StatementPreview, statementPreviewKey, useAccounts, useReadAgain, useStatementPreview } from "@/api/finance";
import { Card, CardContent } from "@/design";
import { confirmedLine } from "./importCopy";
import { unreadCopy } from "../statementCopy";
import { NotReadNotice } from "./NotReadNotice";
import { ReviewBody } from "./ReviewBody";
import { StatementActions } from "./StatementActions";
import { StatementHeader } from "./StatementHeader";

/** One Statement's preview: what was read, where it goes, and Confirm or Discard. */
export function StatementCard({ preview: first }: { preview: StatementPreview }) {
  const client = useQueryClient();
  const [tries, setTries] = useState(0);
  const [confirmed, setConfirmed] = useState<ConfirmCounts | null>(null);
  const [discarded, setDiscarded] = useState(false);
  // Seeded with the upload's preview, then refreshed after every confirm so a section whose
  // Account was just created shows that Account instead of proposing it again.
  const live = useStatementPreview(first.id, !confirmed && !discarded, first);
  const preview = live.data ?? first;
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
                  client.setQueryData(statementPreviewKey(first.id), result.statement);
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
