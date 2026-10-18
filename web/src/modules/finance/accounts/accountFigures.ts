import type { AccountSummary } from "@/api/finance";
import { formatPeriod } from "../dates";
import { formatCount, formatInr } from "../money";

/** The labelled figures an Account card shows, derived from its summary. */
export function accountFigures(account: AccountSummary): { label: string; value: string }[] {
  const transactions = `${formatCount(account.transaction_count)} ${account.transaction_count === 1 ? "Transaction" : "Transactions"}`;
  const statements = `${formatCount(account.statement_count)} ${account.statement_count === 1 ? "Statement" : "Statements"}`;
  return [
    { label: "Latest balance", value: account.latest_balance_paise === null || account.latest_balance_paise === undefined ? "Not printed" : formatInr(account.latest_balance_paise) },
    { label: "Transactions", value: transactions },
    {
      label: "Coverage",
      value: account.first_date ? formatPeriod({ start: account.first_date, end: account.last_date }) : "No Transactions yet",
    },
    { label: "Statements", value: statements },
  ];
}
