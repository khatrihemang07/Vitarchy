import { useAccounts, useConfirmedStatements } from "@/api/finance";
import { CardSkeleton, FadeUp, Notice, QueryNotices } from "@/design";
import { errorMessage } from "../failureMessage";
import { pendingTogether } from "../pendingTogether";
import { AccountCard } from "./AccountCard";
import { splitHidden } from "./hiddenGroup";
import { HiddenAccounts } from "./HiddenAccounts";
import { StatementsCard } from "./StatementsCard";

/** Header and tabs are 0 and 1 of the page's entrance sequence; the Accounts and Statements follow. */
const FIRST_STAGGER = 2;

const skeleton = (
  <div className="flex flex-col gap-6">
    <div className="grid gap-6 md:grid-cols-2">
      <CardSkeleton label="Loading Accounts" lines={4} />
      <CardSkeleton label="Loading Accounts" lines={4} />
    </div>
    <CardSkeleton label="Loading Statements" lines={3} />
  </div>
);

/** `/finance/accounts`: every Account with its figures, and the Statements behind them. */
export function AccountsPage() {
  const accounts = useAccounts();
  const { shown, hidden } = splitHidden(accounts.data ?? []);
  // One skeleton for the whole page until both lists have answered, instead of one spinner per section.
  const loading = pendingTogether([accounts, useConfirmedStatements()]);
  if (loading.isPending) return <QueryNotices query={loading} describeError={errorMessage} skeleton={skeleton} />;
  return (
    <div className="flex flex-col gap-6">
      <FadeUp index={FIRST_STAGGER}>
        {!accounts.data ? (
          <Notice kind="failure" title={errorMessage(accounts.error)} onRetry={() => void accounts.refetch()} />
        ) : accounts.data.length === 0 ? (
          <Notice kind="empty" title="No Accounts yet">
            Confirm a Statement on the Import page and its Accounts appear here.
          </Notice>
        ) : (
          <div className="grid gap-6 md:grid-cols-2">
            {shown.map((account) => (
              <AccountCard key={account.id} account={account} />
            ))}
          </div>
        )}
      </FadeUp>
      <FadeUp index={FIRST_STAGGER}>
        <HiddenAccounts accounts={hidden} />
      </FadeUp>
      <FadeUp index={FIRST_STAGGER + 2}>
        <StatementsCard />
      </FadeUp>
    </div>
  );
}
