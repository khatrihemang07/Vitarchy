import { createRoute, lazyRouteComponent } from "@tanstack/react-router";
import { Wallet } from "lucide-react";
import type { ModuleManifest } from "@/app/module-manifest";
import { FINANCE_COMMAND_PAGES } from "./financePages";
import { FinancePageLayout } from "./FinancePageLayout";
import { parseTransactionsSearch, transactionsParams } from "./transactionsSearch";

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
        createRoute({ getParentRoute: () => layout, path: "/finance", component: ImportPage }),
        createRoute({ getParentRoute: () => layout, path: "/finance/accounts", component: AccountsPage }),
        createRoute({
          getParentRoute: () => layout,
          path: "/finance/transactions",
          // The URL is user-editable: anything unusable falls back to a default.
          validateSearch: (raw: Record<string, unknown>) => transactionsParams(parseTransactionsSearch(raw)),
          component: TransactionsPage,
        }),
      ]),
    ];
  },
  navItem: { to: "/finance" },
  pages: FINANCE_COMMAND_PAGES,
};
