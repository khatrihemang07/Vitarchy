import { useAccounts, useTransactions } from "@/api/finance";
import { ClearFiltersButton, cn, FadeUp, Notice, QueryNotices, SectionCard, TableSkeleton } from "@/design";
import { errorMessage } from "../failureMessage";
import { toTransactionQuery } from "../transactionsSearch";
import { Pagination } from "./Pagination";
import { TransactionFilters } from "./TransactionFilters";
import { TransactionTable } from "./TransactionTable";
import { useTransactionsSearch } from "./useTransactionsSearch";

/** Header and tabs are 0 and 1 of the page's entrance sequence; the filters follow, then the table. */
const FILTERS_STAGGER = 2;
const BODY_STAGGER = FILTERS_STAGGER + 1;

const skeleton = (
  <SectionCard title="Transactions">
    <TableSkeleton label="Loading Transactions" rows={8} columns={4} />
  </SectionCard>
);

/** `/finance/transactions`: Transactions filtered by Account and dates, a page at a time. */
export function TransactionsPage() {
  const { search, update } = useTransactionsSearch();
  const accounts = useAccounts();
  const page = useTransactions(toTransactionQuery(search));
  const filtered = search.account !== undefined || search.from !== undefined || search.to !== undefined;
  // Rows show Account names, so wait for them too: no "Account" placeholder that is replaced a moment later.
  const items = page.data && !accounts.isPending ? page.data : undefined;
  const waitingOn = page.data ? accounts : page;
  return (
    <div className="flex flex-col gap-6">
      <FadeUp index={FILTERS_STAGGER}>
        <div className="flex flex-col gap-3">
          <TransactionFilters search={search} accounts={accounts.data ?? []} onChange={update} />
          {accounts.isError ? (
            <Notice inline kind="failure" title={`Could not load the Accounts to filter by. ${errorMessage(accounts.error)}`} onRetry={() => void accounts.refetch()} />
          ) : null}
        </div>
      </FadeUp>
      {!items ? (
        <QueryNotices query={waitingOn} describeError={errorMessage} skeleton={skeleton} />
      ) : (
        <FadeUp index={BODY_STAGGER}>
          {items.items.length === 0 ? (
            <Notice
              kind="empty"
              title={filtered ? "No Transactions match these filters" : "No Transactions yet"}
              action={filtered ? <ClearFiltersButton onClear={() => { update({}); }} /> : undefined}
            >
              {filtered ? "Try a wider range of dates or another Account." : "Confirm a Statement on the Import page and its Transactions appear here."}
            </Notice>
          ) : (
            <div className="flex flex-col gap-6">
              {page.isError ? <Notice kind="failure" title="Could not load the Transactions" onRetry={() => void page.refetch()} /> : null}
              <div aria-busy={page.isPlaceholderData} className={cn("transition-opacity", page.isPlaceholderData && "opacity-60")}>
                <TransactionTable items={items.items} accounts={accounts.data ?? []} />
              </div>
              <Pagination
                total={items.total}
                page={search.page ?? 1}
                busy={page.isPlaceholderData}
                onPage={(next) => { update({ ...search, page: next }); }}
              />
            </div>
          )}
        </FadeUp>
      )}
    </div>
  );
}
