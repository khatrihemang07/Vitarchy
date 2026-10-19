import { useState } from "react";
import { type TransactionItem, useLinkTransfer, useTransferCandidates } from "@/api/finance";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, MotionButton, Notice, QueryNotices } from "@/design";
import { errorMessage } from "../failureMessage";
import { formatInr } from "../money";
import { CandidateList } from "./CandidateList";
import { linkRequest, selectedPaise, toggleCandidate } from "./candidateSelection";

interface LinkTransferDialogProps {
  item: TransactionItem;
  names: ReadonlyMap<number, string>;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** Offers the Transactions a row could be the other side of and links the chosen ones as a Transfer. */
export function LinkTransferDialog({ item, names, open, onOpenChange }: LinkTransferDialogProps) {
  const [selected, setSelected] = useState<number[]>([]);
  const candidates = useTransferCandidates(item.id, open);
  const link = useLinkTransfer();
  const several = item.direction === "debit";
  const found = candidates.data ?? [];
  const close = (next: boolean) => {
    if (!next) { setSelected([]); link.reset(); }
    onOpenChange(next);
  };
  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Link as Transfer</DialogTitle>
          <DialogDescription>
            {several
              ? "Choose the Money in on other Accounts that this Money out paid. Money that has not arrived yet stays counted as Money out."
              : "Choose the Money out on another Account that paid this Money in."}
          </DialogDescription>
        </DialogHeader>
        {!candidates.data ? (
          <QueryNotices inline query={candidates} describeError={errorMessage} loadingTitle="Loading candidates…" />
        ) : found.length === 0 ? (
          <Notice inline kind="empty" title="Nothing nearby fits">
            No other Account has {several ? "Money in" : "Money out"} within a week of this that is not already part of a Transfer.
          </Notice>
        ) : (
          <CandidateList candidates={found} names={names} several={several} selected={selected} onToggle={(id) => { setSelected((s) => toggleCandidate(s, id, several)); }} />
        )}
        {several && selected.length > 0 ? (
          <p className="text-sm text-muted-foreground">Chosen: {formatInr(selectedPaise(found, selected))} of {formatInr(item.amount_paise)}</p>
        ) : null}
        {link.isError ? <p role="alert" className="text-sm text-destructive">{errorMessage(link.error)}</p> : null}
        <DialogFooter>
          <MotionButton variant="outline" onClick={() => { close(false); }}>Cancel</MotionButton>
          <MotionButton
            disabled={selected.length === 0 || link.isPending}
            onClick={() => { link.mutate(linkRequest(item, selected), { onSuccess: () => { close(false); } }); }}
          >
            {link.isPending ? "Linking…" : "Link"}
          </MotionButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
