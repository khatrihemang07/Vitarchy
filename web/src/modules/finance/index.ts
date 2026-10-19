import { createRoute, lazyRouteComponent } from "@tanstack/react-router";
import { Wallet } from "lucide-react";
import {
  accountsOptions,
  confirmedStatementsOptions,
  financeStatusOptions,
  passwordsOptions,
  unconfirmedStatementsOptions,
} from "@/api/finance";
import { type ModuleManifest, warm, warmCache, warmCacheWith } from "@/app/module-manifest";
import { FINANCE_COMMAND_PAGES } from "./financePages";
import { FinancePageLayout } from "./FinancePageLayout";
import { warmTransactions } from "./transactions/transactionsLoader";
import { parseTransactionsSearch, transactionsParams } from "./transactions/transactionsSearch";

// Pages load on first visit.
const ImportPage = lazyRouteComponent(() => import("./import/ImportPage"), "ImportPage");
const AccountsPage = lazyRouteComponent(() => import("./accounts/AccountsPage"), "AccountsPage");
const TransactionsPage = lazyRouteComponent(() => import("./transactions/TransactionsPage"), "TransactionsPage");

/** The Finance Module: Statements, Accounts and Transactions. */
export const financeModule: ModuleManifest = {
  id: "finance",
  label: "Finance",
  accent: "var(--module-finance)",
  icon: Wallet,
  routes: (root) => {
    // One layout route holds the header and tabs, so they persist between the pages.
    const layout = createRoute({ getParentRoute: () => root, id: "finance-layout", component: FinancePageLayout });
    return [
      layout.addChildren([
        createRoute({
          getParentRoute: () => layout,
          path: "/finance",
          // Loaders warm the cache, so hovering a tab (or arriving at it) finds its data already loading.
          loader: warmCache([
            warm(financeStatusOptions),
            warm(unconfirmedStatementsOptions),
            warm(passwordsOptions),
          ]),
          component: ImportPage,
        }),
        createRoute({
          getParentRoute: () => layout,
          path: "/finance/accounts",
          loader: warmCache([
            warm(accountsOptions),
            warm(confirmedStatementsOptions),
          ]),
          component: AccountsPage,
        }),
        createRoute({
          getParentRoute: () => layout,
          path: "/finance/transactions",
          // The URL is user-editable: anything unusable falls back to a default.
          validateSearch: (raw: Record<string, unknown>) => transactionsParams(parseTransactionsSearch(raw)),
          // The page's query depends on the filters in the address, so the loader re-runs when they change.
          loaderDeps: ({ search }: { search: Record<string, unknown> }) => ({ search }),
          loader: warmCacheWith(({ search }: { search: Record<string, unknown> }) => [warmTransactions(search)]),
          component: TransactionsPage,
        }),
      ]),
    ];
  },
  navItem: { to: "/finance" },
  pages: FINANCE_COMMAND_PAGES,
};
