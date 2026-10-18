import { Check, Pencil, X } from "lucide-react";
import { type SyntheticEvent, useState } from "react";
import { type AccountSummary, useRenameAccount } from "@/api/finance";
import { Input, MotionButton } from "@/design";
import { errorMessage } from "../failureMessage";

/** An Account's name that turns into a field when edited, saved with Enter and left with Escape. */
export function RenameField({ account }: { account: AccountSummary }) {
  const [draft, setDraft] = useState<string | null>(null);
  const rename = useRenameAccount();

  if (draft === null) {
    return (
      <div className="flex items-center gap-1">
        <h3 className="font-display text-lg font-semibold break-words">{account.name}</h3>
        <MotionButton
          variant="ghost"
          size="icon"
          aria-label={`Rename ${account.name}`}
          onClick={() => { rename.reset(); setDraft(account.name); }}
        >
          <Pencil aria-hidden="true" className="size-4" />
        </MotionButton>
      </div>
    );
  }

  const save = (event: SyntheticEvent) => {
    event.preventDefault();
    rename.mutate({ id: account.id, name: draft.trim() }, { onSuccess: () => { setDraft(null); } });
  };
  return (
    <form onSubmit={save} className="flex flex-col gap-1.5">
      <div className="flex items-center gap-1">
        <Input
          autoFocus
          aria-label="Account name"
          value={draft}
          onChange={(event) => { setDraft(event.target.value); }}
          onKeyDown={(event) => { if (event.key === "Escape") setDraft(null); }}
        />
        <MotionButton type="submit" variant="ghost" size="icon" aria-label="Save name" disabled={rename.isPending || draft.trim() === ""}>
          <Check aria-hidden="true" className="size-4" />
        </MotionButton>
        <MotionButton type="button" variant="ghost" size="icon" aria-label="Cancel rename" onClick={() => { setDraft(null); }}>
          <X aria-hidden="true" className="size-4" />
        </MotionButton>
      </div>
      {rename.isError ? <p role="alert" className="text-sm text-destructive">{errorMessage(rename.error)}</p> : null}
    </form>
  );
}
