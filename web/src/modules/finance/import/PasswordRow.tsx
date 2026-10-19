import { useState } from "react";
import { type PasswordItem, useDeletePassword } from "@/api/finance";
import { ListRow, MotionButton } from "@/design";
import { formatLastUsed } from "../dates";
import { errorMessage } from "../failureMessage";

/** One saved Statement password: its label and when it last opened a file, with Remove behind a confirmation. */
export function PasswordRow({ password }: { password: PasswordItem }) {
  const [asking, setAsking] = useState(false);
  const remove = useDeletePassword();
  return (
    <ListRow title={password.label} detail={formatLastUsed(password.last_used_at)}>
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
    </ListRow>
  );
}
