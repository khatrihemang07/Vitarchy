import type { StatementPreview } from "@/api/finance";
import { MotionButton, Notice } from "@/design";
import { errorMessage } from "../failureMessage";
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
      <Notice kind="warning" title={copy.title} inline>
        {copy.body}
        {outcome === "locked" ? <PasswordsLink /> : null}
      </Notice>
      {retryError ? <Notice kind="failure" inline title={errorMessage(retryError)} /> : null}
      <MotionButton variant="outline" size="sm" className="w-fit" onClick={onTryAgain} disabled={retrying}>
        {retrying ? "Trying again…" : "Try again"}
      </MotionButton>
    </div>
  );
}
