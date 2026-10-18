import { type AccountSummary, type ConfirmCounts, type StatementPreview, useConfirmStatement } from "@/api/finance";
import { CardContent } from "@/design";
import { PaymentHistorySummary } from "./PaymentHistorySummary";
import { SectionView } from "./SectionView";
import { StatementActions } from "./StatementActions";
import { useChoiceDrafts } from "./useChoiceDrafts";

interface ReviewBodyProps {
  preview: StatementPreview;
  accounts: readonly AccountSummary[];
  onConfirmed: (counts: ConfirmCounts) => void;
  onDiscarded: () => void;
}

/** The sections of a read Statement with an Account choice each, and Confirm and Discard. */
export function ReviewBody({ preview, accounts, onConfirmed, onDiscarded }: ReviewBodyProps) {
  const { drafts, setDraft, ready, choices } = useChoiceDrafts(preview.sections);
  const confirm = useConfirmStatement();
  const alreadyConfirmed = preview.status === "confirmed";
  return (
    <CardContent className="flex flex-col gap-4">
      {alreadyConfirmed ? <p className="text-sm text-muted-foreground">This file was uploaded before and is already confirmed.</p> : null}
      {preview.payment_history ? <PaymentHistorySummary history={preview.payment_history} /> : null}
      {preview.sections.map((section, index) => (
        <SectionView
          key={section.position}
          section={section}
          accounts={accounts}
          draft={drafts[index] ?? { mode: "existing", accountId: null }}
          onDraft={(next) => { setDraft(index, next); }}
          locked={alreadyConfirmed}
        />
      ))}
      {alreadyConfirmed ? null : (
        <StatementActions
          statementId={preview.id}
          confirmDisabled={!ready}
          confirming={confirm.isPending}
          confirmError={confirm.error}
          onConfirm={() => { confirm.mutate({ id: preview.id, sections: choices() }, { onSuccess: onConfirmed }); }}
          onDiscarded={onDiscarded}
        />
      )}
    </CardContent>
  );
}
