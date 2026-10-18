/** Each Finance page keyed by its path: its tab label and heading, so the tabs and the shared frame derive from one place. */
export const FINANCE_PAGES = {
  "/finance": { tab: "Import", title: "Finance", subtitle: "Import Statements from your banks and card issuers." },
  "/finance/accounts": { tab: "Accounts", title: "Accounts", subtitle: "Your Accounts, and the Statements behind them." },
  "/finance/transactions": { tab: "Transactions", title: "Transactions", subtitle: "Every Transaction your Statements printed." },
} as const;

type FinancePagePath = keyof typeof FINANCE_PAGES;

/** The heading for a pathname; anything unknown (a trailing slash, a future page) falls back to the Import page. */
export function financePageFor(pathname: string): (typeof FINANCE_PAGES)[FinancePagePath] {
  const path = pathname.replace(/\/+$/, "") || "/";
  return path in FINANCE_PAGES ? FINANCE_PAGES[path as FinancePagePath] : FINANCE_PAGES["/finance"];
}

/** The tabs of the Finance pages, in page order. */
export const FINANCE_TABS = (Object.keys(FINANCE_PAGES) as FinancePagePath[]).map((to) => ({ to, label: FINANCE_PAGES[to].tab }));

/** The Finance pages as the command bar lists them ("Finance: Accounts"), derived from the same table. */
export const FINANCE_COMMAND_PAGES = (Object.keys(FINANCE_PAGES) as FinancePagePath[]).map((to) => ({
  id: FINANCE_PAGES[to].tab.toLowerCase(),
  label: FINANCE_PAGES[to].tab,
  to,
}));
