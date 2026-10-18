import { useAccounts } from "@/api/finance";
import { FinanceNotice } from "../FinanceNotice";
import { QueryNotices } from "../QueryNotices";
import { AccountCard } from "./AccountCard";
import { StatementsCard } from "./StatementsCard";

/** `/finance/accounts`: every Account with its figures, and the Statements behind them. */
export function AccountsPage() {
  const accounts = useAccounts();
  return (
    <div className="flex flex-col gap-6">
      {!accounts.data ? (
        <QueryNotices query={accounts} loadingTitle="Loading Accounts…" />
      ) : accounts.data.length === 0 ? (
        <FinanceNotice kind="empty" title="No Accounts yet">
          Confirm a Statement on the Import page and its Accounts appear here.
        </FinanceNotice>
      ) : (
        <div className="grid gap-6 md:grid-cols-2">
          {accounts.data.map((account) => (
            <AccountCard key={account.id} account={account} />
          ))}
        </div>
      )}
      <StatementsCard />
    </div>
  );
}
