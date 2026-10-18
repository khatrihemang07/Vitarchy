import type { StatementPreview } from "@/api/finance";
import { errorMessage } from "../failureMessage";
import { FinanceNotice } from "../FinanceNotice";
import { type UnreadCopy } from "../statementCopy";
import { PasswordsLink } from "./PasswordsLink";

interface NotReadNoticeProps {
  copy: UnreadCopy;
  outcome: StatementPreview["outcome"];
  retrying: boolean;
  retryError: unknown;
  onTryAgain: () => void;
}

/** Why a file was not read, in plain words, with Try again (Read again) and, for a locked file, a link to the passwords. */
export function NotReadNotice({ copy, outcome, retrying, retryError, onTryAgain }: NotReadNoticeProps) {
  return (
    <div className="flex flex-col gap-3">
      <FinanceNotice kind="warning" title={copy.title} inline>
        {copy.body}
        {outcome === "locked" ? <PasswordsLink /> : null}
      </FinanceNotice>
      {retryError ? <FinanceNotice kind="failure" inline title={errorMessage(retryError)} /> : null}
      <button
        type="button"
        onClick={onTryAgain}
        disabled={retrying}
        className="w-fit rounded-full border px-4 py-1.5 text-sm font-medium transition-colors hover:bg-accent focus-ring disabled:opacity-50"
      >
        {retrying ? "Trying again…" : "Try again"}
      </button>
    </div>
  );
}
