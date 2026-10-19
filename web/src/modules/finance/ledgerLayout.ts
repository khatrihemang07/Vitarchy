import type { ColumnHeading } from "@/design";

/**
 * Rows of a Transactions table are stacked cards on narrow screens and grid tracks from `md`.
 * The tracks come from the `--track-*` tokens through the `grid-cols-ledger*` utilities, so
 * widths live in one place; the header bar and every row share the grid, so columns line up.
 */
const ROW = "md:grid md:items-start";

/** The Transactions page: Date, Account, Description, Amount, Balance. */
export const TRANSACTION_GRID = `${ROW} grid-cols-2 md:grid-cols-ledger`;

/** A Statement's rows: the same tracks, without the Account. */
export const STATEMENT_ROW_GRID = `${ROW} grid-cols-2 md:grid-cols-ledger-statement`;

/** What every row has in common: spacing and the gap between cells. */
export const LEDGER_ROW = "grid gap-x-3 gap-y-1 px-4 py-3";

/**
 * Where each cell sits. Below `md` the date and Account stack on the left, the amount (with the
 * balance under it) on the right, and the description takes the row below; from `md` cells
 * simply flow into their tracks.
 */
export const LEDGER_CELLS = {
  date: "whitespace-nowrap max-md:col-start-1 max-md:row-start-1",
  account: "text-xs text-muted-foreground max-md:col-start-1 max-md:row-start-2 md:text-sm",
  description: "min-w-0 [overflow-wrap:anywhere] max-md:col-span-2",
  amount: "justify-self-end text-right whitespace-nowrap max-md:col-start-2 max-md:row-span-2 max-md:row-start-1",
  balance: "hidden text-right whitespace-nowrap text-muted-foreground tabular-nums md:block",
  balanceUnderAmount: "block text-xs text-muted-foreground tabular-nums md:hidden",
  actions: "justify-self-end max-md:col-span-2",
} as const;

const DATE: ColumnHeading = { id: "date", label: "Date", tooltip: "When the Statement says it happened." };
const DESCRIPTION: ColumnHeading = { id: "description", label: "Description", tooltip: "As the Statement printed it." };
const AMOUNT: ColumnHeading = {
  id: "amount",
  label: "Amount",
  tooltip: "Money in is green and starts with +; Money out is red and starts with −.",
  align: "end",
};
const BALANCE: ColumnHeading = { id: "balance", label: "Balance", tooltip: "What the Statement printed as the balance after it, when it printed one.", align: "end" };

/** The header bar of the Transactions page, one heading per grid track. */
export const TRANSACTION_HEADINGS: readonly ColumnHeading[] = [
  DATE,
  { id: "account", label: "Account", tooltip: "The Account the money moved on." },
  DESCRIPTION,
  AMOUNT,
  BALANCE,
  { id: "actions", label: "" },
];

/** The header bar of a Statement's rows. */
export const STATEMENT_ROW_HEADINGS: readonly ColumnHeading[] = [DATE, DESCRIPTION, AMOUNT, BALANCE];
