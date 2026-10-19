import { type ReadAgainCounts, type ReadAgainPreview, useConfirmReadAgain } from "@/api/finance";
import { MotionButton, Notice } from "@/design";
import { errorMessage } from "../failureMessage";
import { appliedLine } from "./readAgainCopy";
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
            {appliedLine(applied)}.
          </span>
        </p>
      ) : (
        <ReadAgainChanges preview={preview} />
      )}
      {apply.isError ? <Notice kind="failure" inline title={errorMessage(apply.error)} /> : null}
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
