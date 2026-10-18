import { useAccounts, useTransactions } from "@/api/finance";
import { Button, Card, CardContent } from "@/design";
import { errorMessage } from "../failureMessage";
import { FinanceNotice } from "../FinanceNotice";
import { QueryNotices } from "../QueryNotices";
import { toTransactionQuery } from "../transactionsSearch";
import { Pagination } from "./Pagination";
import { TransactionFilters } from "./TransactionFilters";
import { TransactionTable } from "./TransactionTable";
import { useTransactionsSearch } from "./useTransactionsSearch";

/** `/finance/transactions`: Transactions filtered by Account and dates, a page at a time. */
export function TransactionsPage() {
  const { search, update } = useTransactionsSearch();
  const accounts = useAccounts();
  const page = useTransactions(toTransactionQuery(search));
  const filtered = search.account !== undefined || search.from !== undefined || search.to !== undefined;
  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardContent>
          <TransactionFilters search={search} accounts={accounts.data ?? []} onChange={update} />
          {accounts.isError ? (
            <FinanceNotice inline kind="failure" title={`Could not load the Accounts to filter by. ${errorMessage(accounts.error)}`} onRetry={() => void accounts.refetch()} />
          ) : null}
        </CardContent>
      </Card>
      {!page.data ? (
        <QueryNotices query={page} loadingTitle="Loading Transactions…" />
      ) : page.data.items.length === 0 ? (
        <FinanceNotice
          kind="empty"
          title={filtered ? "No Transactions match these filters" : "No Transactions yet"}
          action={filtered ? <Button variant="link" onClick={() => { update({}); }}>Clear filters</Button> : undefined}
        >
          {filtered ? "Try a wider range of dates or another Account." : "Confirm a Statement on the Import page and its Transactions appear here."}
        </FinanceNotice>
      ) : (
        <>
          {page.isError ? <FinanceNotice kind="failure" title="Could not load the Transactions" onRetry={() => void page.refetch()} /> : null}
          <div aria-busy={page.isPlaceholderData} className={page.isPlaceholderData ? "opacity-60 transition-opacity" : undefined}>
            <TransactionTable items={page.data.items} accounts={accounts.data ?? []} />
          </div>
          <Pagination
            total={page.data.total}
            page={search.page ?? 1}
            busy={page.isPlaceholderData}
            onPage={(next) => { update({ ...search, page: next }); }}
          />
        </>
      )}
    </div>
  );
}
