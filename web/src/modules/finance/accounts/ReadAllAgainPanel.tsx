import { type ReadAgainItem, type ReadAllAgainCounts, useConfirmReadAllAgain } from "@/api/finance";
import { MotionButton } from "@/design";
import { errorMessage } from "../failureMessage";
import { FinanceNotice } from "../FinanceNotice";
import { formatCount } from "../money";
import { summariseReadAllAgain } from "./readAllAgainSummary";

interface ReadAllAgainPanelProps {
  items: ReadAgainItem[];
  applied: ReadAllAgainCounts | null;
  onApplied: (counts: ReadAllAgainCounts) => void;
  onClose: () => void;
}

/** The combined preview of Read all again: totals, the Statements with changes, Apply and Cancel. */
export function ReadAllAgainPanel({ items, applied, onApplied, onClose }: ReadAllAgainPanelProps) {
  const apply = useConfirmReadAllAgain();
  const summary = summariseReadAllAgain(items);
  return (
    <div className="flex flex-col gap-3 rounded-xl border bg-muted/30 p-4">
      {applied ? (
        <p role="status" className="text-sm">
          <span className="font-medium">Read all again applied.</span>{" "}
          <span className="text-muted-foreground">
            {formatCount(applied.statements)} Statements changed: {formatCount(applied.added)} added, {formatCount(applied.no_longer_found)} no longer found
            {applied.linked > 0 ? `, ${formatCount(applied.linked)} Payment records linked` : ""}
            {applied.failed > 0 ? `. ${formatCount(applied.failed)} could not be read again` : ""}.
          </span>
        </p>
      ) : items.length === 0 ? (
        <FinanceNotice kind="empty" inline title="No Statement has its original stored, so there is nothing to read again." />
      ) : (
        <>
          <p className="text-sm">
            <span className="font-medium">{formatCount(summary.changed.length)} of {formatCount(items.length)} Statements would change.</span>{" "}
            <span className="text-muted-foreground">
              {formatCount(summary.add)} to add, {formatCount(summary.unchanged)} unchanged, {formatCount(summary.noLongerFound)} no longer found (marked, never deleted).
            </span>
          </p>
          {summary.changed.length > 0 ? (
            <ul aria-label="Statements with changes" className="flex flex-col gap-1 text-sm">
              {summary.changed.map((c) => (
                <li key={c.statementId} className="flex flex-wrap justify-between gap-x-3">
                  <span className="break-all font-medium">{c.fileName}</span>
                  <span className="text-muted-foreground">{formatCount(c.add)} to add, {formatCount(c.noLongerFound)} no longer found</span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-sm text-muted-foreground">Nothing has changed since these Statements were read.</p>
          )}
          {summary.failed.length > 0 ? (
            <ul aria-label="Statements that could not be read again" className="flex flex-col gap-1 text-sm">
              {summary.failed.map((f) => (
                <li key={f.statementId}>
                  <span className="break-all font-medium">{f.fileName}</span> <span className="text-muted-foreground">{f.reason}</span>
                </li>
              ))}
            </ul>
          ) : null}
        </>
      )}
      {apply.isError ? <FinanceNotice kind="failure" inline title={errorMessage(apply.error)} /> : null}
      <div className="flex flex-wrap gap-2">
        {applied || summary.changed.length === 0 ? null : (
          <MotionButton disabled={apply.isPending} onClick={() => { apply.mutate(undefined, { onSuccess: onApplied }); }}>
            {apply.isPending ? "Applying…" : "Apply all changes"}
          </MotionButton>
        )}
        <MotionButton variant="outline" onClick={onClose}>
          {applied ? "Close" : "Cancel"}
        </MotionButton>
      </div>
    </div>
  );
}
