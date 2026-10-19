import { type AccountSummary, useSetAccountHidden } from "@/api/finance";
import { MotionButton } from "@/design";
import { errorMessage } from "../failureMessage";

/** Hides an Account (its Transactions leave the Transactions page and it takes no part in Transfers) or shows it again. */
export function HideAccountButton({ account }: { account: AccountSummary }) {
  const setHidden = useSetAccountHidden();
  return (
    <div className="flex flex-col gap-1">
      <MotionButton
        variant="outline"
        size="sm"
        className="w-fit"
        aria-label={`${account.hidden ? "Unhide" : "Hide"} ${account.name}`}
        disabled={setHidden.isPending}
        onClick={() => { setHidden.mutate({ id: account.id, hidden: !account.hidden }); }}
      >
        {account.hidden ? "Unhide" : "Hide"}
      </MotionButton>
      {setHidden.isError ? <p role="alert" className="text-sm text-destructive">{errorMessage(setHidden.error)}</p> : null}
    </div>
  );
}
