import { useState } from "react";
import { type PasswordItem, useDeletePassword } from "@/api/finance";
import { MotionButton } from "@/design";
import { formatLastUsed } from "../dates";
import { errorMessage } from "../failureMessage";

/** One saved Statement password: its label and when it last opened a file, with Remove behind a confirmation. */
export function PasswordRow({ password }: { password: PasswordItem }) {
  const [asking, setAsking] = useState(false);
  const remove = useDeletePassword();
  return (
    <li className="flex flex-wrap items-center gap-x-3 gap-y-2 rounded-xl border bg-card p-3">
      <div className="flex min-w-40 flex-1 flex-col">
        <span className="font-medium break-words">{password.label}</span>
        <span className="text-sm text-muted-foreground">{formatLastUsed(password.last_used_at)}</span>
      </div>
      {remove.isError ? <span role="alert" className="text-sm text-destructive">{errorMessage(remove.error)}</span> : null}
      {asking ? (
        <div className="flex items-center gap-2">
          <span className="text-sm">Remove this password?</span>
          <MotionButton
            variant="destructive"
            size="sm"
            disabled={remove.isPending}
            onClick={() => { remove.mutate(password.id); }}
          >
            {remove.isPending ? "Removing…" : "Remove"}
          </MotionButton>
          <MotionButton variant="outline" size="sm" onClick={() => { setAsking(false); }}>
            Keep
          </MotionButton>
        </div>
      ) : (
        <MotionButton variant="outline" size="sm" aria-label={`Remove ${password.label}`} onClick={() => { setAsking(true); }}>
          Remove
        </MotionButton>
      )}
    </li>
  );
}
