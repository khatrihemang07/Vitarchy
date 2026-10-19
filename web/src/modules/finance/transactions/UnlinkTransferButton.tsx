import { useUnlinkTransfer } from "@/api/finance";
import { MotionButton } from "@/design";
import { errorMessage } from "../failureMessage";

/** Unlinks a Transfer; detection will not link the same pair again. */
export function UnlinkTransferButton({ transferId }: { transferId: number }) {
  const unlink = useUnlinkTransfer();
  return (
    <div className="flex flex-col gap-2">
      <MotionButton variant="outline" size="sm" disabled={unlink.isPending} onClick={() => { unlink.mutate(transferId); }}>
        {unlink.isPending ? "Unlinking…" : "Unlink"}
      </MotionButton>
      {unlink.isError ? <p role="alert" className="text-sm text-destructive">{errorMessage(unlink.error)}</p> : null}
    </div>
  );
}
