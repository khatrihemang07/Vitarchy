import { useAccounts } from "@/api/finance";
import { FadeUp, Notice, QueryNotices } from "@/design";
import { errorMessage } from "../failureMessage";
import { AccountCard } from "./AccountCard";
import { splitHidden } from "./hiddenGroup";
import { HiddenAccounts } from "./HiddenAccounts";
import { StatementsCard } from "./StatementsCard";

/** Header and tabs are 0 and 1 of the page's entrance sequence; the Accounts and Statements follow. */
const FIRST_STAGGER = 2;

/** `/finance/accounts`: every Account with its figures, and the Statements behind them. */
export function AccountsPage() {
  const accounts = useAccounts();
  const { shown, hidden } = splitHidden(accounts.data ?? []);
  return (
    <div className="flex flex-col gap-6">
      {!accounts.data ? (
        <QueryNotices query={accounts} describeError={errorMessage} loadingTitle="Loading Accounts…" stagger={FIRST_STAGGER} />
      ) : (
        <FadeUp index={FIRST_STAGGER}>
          {accounts.data.length === 0 ? (
            <Notice kind="empty" title="No Accounts yet">
              Confirm a Statement on the Import page and its Accounts appear here.
            </Notice>
          ) : (
            <div className="flex flex-col gap-6">
              <div className="grid gap-6 md:grid-cols-2">
                {shown.map((account) => (
                  <AccountCard key={account.id} account={account} />
                ))}
              </div>
              <HiddenAccounts accounts={hidden} />
            </div>
          )}
        </FadeUp>
      )}
      <FadeUp index={FIRST_STAGGER + 1}>
        <StatementsCard />
      </FadeUp>
    </div>
  );
}
