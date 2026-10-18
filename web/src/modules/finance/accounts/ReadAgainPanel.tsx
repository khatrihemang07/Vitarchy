import { type ReadAgainCounts, type ReadAgainPreview, useConfirmReadAgain } from "@/api/finance";
import { MotionButton } from "@/design";
import { errorMessage } from "../failureMessage";
import { FinanceNotice } from "../FinanceNotice";
import { formatCount } from "../money";
import { ReadAgainChanges } from "./ReadAgainChanges";

interface ReadAgainPanelProps {
  preview: ReadAgainPreview;
  applied: ReadAgainCounts | null;
  onApplied: (counts: ReadAgainCounts) => void;
  onClose: () => void;
}

/** The preview of a Read again with Apply and Cancel; after Apply, what it did. */
export function ReadAgainPanel({ preview, applied, onApplied, onClose }: ReadAgainPanelProps) {
  const apply = useConfirmReadAgain();
  return (
    <div className="flex flex-col gap-3 rounded-xl border bg-muted/30 p-4">
      {applied ? (
        <p role="status" className="text-sm">
          <span className="font-medium">Read again applied.</span>{" "}
          <span className="text-muted-foreground">
            {formatCount(applied.added)} added, {formatCount(applied.unchanged)} unchanged, {formatCount(applied.no_longer_found)} no longer found
            {applied.linked > 0 ? `, ${formatCount(applied.linked)} Payment records linked` : ""}.
          </span>
        </p>
      ) : (
        <ReadAgainChanges preview={preview} />
      )}
      {apply.isError ? <FinanceNotice kind="failure" inline title={errorMessage(apply.error)} /> : null}
      <div className="flex flex-wrap gap-2">
        {applied ? null : (
          <MotionButton
            disabled={apply.isPending}
            onClick={() => { apply.mutate(preview.statement.id, { onSuccess: onApplied }); }}
          >
            {apply.isPending ? "Applying…" : "Apply changes"}
          </MotionButton>
        )}
        <MotionButton variant="outline" onClick={onClose}>
          {applied ? "Close" : "Cancel"}
        </MotionButton>
      </div>
    </div>
  );
}
