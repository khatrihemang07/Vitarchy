import { Notice, QueryNotices, TableSkeleton, CardSkeleton, SectionCard, FadeUp, ClearFiltersButton, cn } from "@/design";
import { errorMessage } from "../failureMessage";
import { OverviewStrip } from "./OverviewStrip";
import { Pagination } from "./Pagination";
import { TransactionsFilterBar } from "./TransactionsFilterBar";
import { TransactionTable } from "./TransactionTable";
import { emptyCopy, emptyReason } from "./emptyCopy";
import { hasActiveFilters } from "./transactionsSearch";
import { useTransactionsView } from "./useTransactionsView";

/** Header and tabs are 0 and 1 of the page's entrance sequence; the filters, the Overview and the table follow. */
const FILTERS_STAGGER = 2;
const OVERVIEW_STAGGER = FILTERS_STAGGER + 1;
const BODY_STAGGER = OVERVIEW_STAGGER + 1;

const tableSkeleton = (
  <SectionCard title="Transactions">
    <TableSkeleton label="Loading Transactions" rows={8} columns={4} />
  </SectionCard>
);

/** `/finance/transactions`: Money in, Money out and Transfers of a Period, and the Transactions behind them, a page at a time. */
export function TransactionsPage() {
  const { search, range, today, handlers, accounts, page, overview } = useTransactionsView();
  const filtered = hasActiveFilters(search);
  // Rows show Account names, so wait for them too: no "Account" placeholder that is replaced a moment later.
  const items = page.data && !accounts.isPending ? page.data : undefined;
  const waitingOn = page.data ? accounts : page;
  const everStored = (accounts.data ?? []).reduce((sum, account) => sum + account.transaction_count, 0) > 0;
  const reason = emptyReason({ filtered, everStored });
  const empty = emptyCopy(reason);
  return (
    <div className="flex flex-col gap-6">
      <FadeUp index={FILTERS_STAGGER}>
        <div className="flex flex-col gap-3">
          <TransactionsFilterBar search={search} range={range} today={today} accounts={accounts.data ?? []} handlers={handlers} />
          {accounts.isError ? (
            <Notice inline kind="failure" title={`Could not load the Accounts to filter by. ${errorMessage(accounts.error)}`} onRetry={() => void accounts.refetch()} />
          ) : null}
        </div>
      </FadeUp>
      <FadeUp index={OVERVIEW_STAGGER}>
        {overview.data ? (
          <div aria-busy={overview.isPlaceholderData} className={cn("transition-opacity", overview.isPlaceholderData && "opacity-60")}>
            <OverviewStrip overview={overview.data} />
          </div>
        ) : (
          <QueryNotices query={overview} describeError={errorMessage} skeleton={<CardSkeleton label="Loading overview" lines={4} />} />
        )}
      </FadeUp>
      {!items ? (
        <QueryNotices query={waitingOn} describeError={errorMessage} skeleton={tableSkeleton} />
      ) : (
        <FadeUp index={BODY_STAGGER}>
          {items.items.length === 0 ? (
            <Notice kind="empty" title={empty.title} action={reason === "filtered" ? <ClearFiltersButton onClear={handlers.onClear} /> : undefined}>
              {empty.hint}
            </Notice>
          ) : (
            <div className="flex flex-col gap-6">
              {page.isError ? <Notice kind="failure" title="Could not load the Transactions" onRetry={() => void page.refetch()} /> : null}
              <div aria-busy={page.isPlaceholderData} className={cn("transition-opacity", page.isPlaceholderData && "opacity-60")}>
                <TransactionTable items={items.items} accounts={accounts.data ?? []} sort={search.sort} onSort={handlers.onSortColumn} />
              </div>
              <Pagination total={items.total} page={search.page ?? 1} busy={page.isPlaceholderData} onPage={handlers.onPage} />
            </div>
          )}
        </FadeUp>
      )}
    </div>
  );
}
