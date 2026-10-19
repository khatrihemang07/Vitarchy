import type { AccountSummary } from "@/api/finance";

/** The Accounts the person keeps in view, and the ones they hid (kept in the Accounts' own order). */
export function splitHidden(accounts: readonly AccountSummary[]): { shown: AccountSummary[]; hidden: AccountSummary[] } {
  return {
    shown: accounts.filter((a) => !a.hidden),
    hidden: accounts.filter((a) => a.hidden),
  };
}
