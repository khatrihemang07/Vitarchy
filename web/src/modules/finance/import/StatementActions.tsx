import { useDiscardStatement } from "@/api/finance";
import { MotionButton } from "@/design";
import { errorMessage } from "../failureMessage";
import { FinanceNotice } from "../FinanceNotice";

interface StatementActionsProps {
  statementId: number;
  /** Present for a read Statement; absent when only Discard makes sense. */
  onConfirm?: () => void;
  confirmDisabled?: boolean;
  confirming?: boolean;
  confirmError?: unknown;
  onDiscarded: () => void;
}

/** Confirm and Discard for one unconfirmed Statement. */
export function StatementActions({ statementId, onConfirm, confirmDisabled, confirming, confirmError, onDiscarded }: StatementActionsProps) {
  const discard = useDiscardStatement();
  const error = confirmError ?? discard.error;
  return (
    <div className="flex flex-col gap-3">
      {error ? <FinanceNotice kind="failure" inline title={errorMessage(error)} /> : null}
      <div className="flex flex-wrap gap-2">
        {onConfirm ? (
          <MotionButton disabled={confirmDisabled === true || confirming === true || discard.isPending} onClick={onConfirm}>
            {confirming ? "Confirming…" : "Confirm"}
          </MotionButton>
        ) : null}
        <MotionButton
          variant="outline"
          disabled={confirming === true || discard.isPending}
          onClick={() => { discard.mutate(statementId, { onSuccess: onDiscarded }); }}
        >
          {discard.isPending ? "Discarding…" : "Discard"}
        </MotionButton>
      </div>
    </div>
  );
}
