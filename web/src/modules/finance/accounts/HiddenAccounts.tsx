import { ChevronRight } from "lucide-react";
import { useId, useState } from "react";
import type { AccountSummary } from "@/api/finance";
import { cn, Expand } from "@/design";
import { AccountCard } from "./AccountCard";

/** The Accounts the person hid, collapsed behind a "Hidden" heading until asked for. */
export function HiddenAccounts({ accounts }: { accounts: readonly AccountSummary[] }) {
  const [open, setOpen] = useState(false);
  const listId = useId();
  if (accounts.length === 0) return null;
  return (
    <section aria-label="Hidden Accounts" className="flex flex-col gap-4">
      <button
        type="button"
        aria-expanded={open}
        aria-controls={listId}
        onClick={() => { setOpen((v) => !v); }}
        className="flex w-fit items-center gap-1 rounded-md font-display text-xl font-bold focus-ring"
      >
        <ChevronRight aria-hidden="true" className={cn("size-5 transition-transform", open && "rotate-90")} />
        Hidden ({accounts.length})
      </button>
      <Expand open={open} id={listId}>
        <div className="grid gap-6 md:grid-cols-2">
          {accounts.map((account) => (
            <AccountCard key={account.id} account={account} />
          ))}
        </div>
      </Expand>
    </section>
  );
}
