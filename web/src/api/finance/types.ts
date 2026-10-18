import type { components, paths } from "../schema";

type Schemas = components["schemas"];

/** A Statement as the person reviews it, before or after confirming. */
export type StatementPreview = Schemas["StatementPreview"];
/** One Account's part of a Statement. */
export type SectionPreview = Schemas["SectionPreview"];
/** A Transaction as a Statement printed it. */
export type TransactionView = Schemas["TransactionView"];
/** What a read file holds: a bank `statement` or a `payment_history`. */
export type StatementKind = Schemas["Kind"];
/** Which way money moved: `debit` or `credit`. */
export type Direction = Schemas["Direction"];
/** A test of a section against itself. */
export type CheckView = Schemas["CheckView"];
/** One line of the Statements list. */
export type StatementSummary = Schemas["StatementSummary"];
/** An Account with its figures. */
export type AccountSummary = Schemas["AccountSummary"];
/** An existing Account by id and name. */
export type AccountRef = Schemas["AccountRef"];
/** What the person chose for one section when confirming. */
export type SectionChoice = Schemas["SectionChoice"];
/** What a confirm did. */
export type ConfirmCounts = Schemas["ConfirmCounts"];
/** A Read again: the Statement and what reading it again would change. */
export type ReadAgainPreview = Schemas["ReadAgainPreview"];
/** The changes a Read again would make to one Account section. */
export type SectionChanges = Schemas["SectionChanges"];
/** What an applied Read again did. */
export type ReadAgainCounts = Schemas["ReadAgainCounts"];
/** One Statement's part of Read all again: its preview, or why it failed. */
export type ReadAgainItem = Schemas["ReadAgainItem"];
/** What an applied Read all again did, summed over the Statements. */
export type ReadAllAgainCounts = Schemas["ReadAllAgainCounts"];
/** What `GET /api/finance/status` returns. */
export type FinanceStatus = Schemas["FinanceStatus"];
/** One read, confirm or Read again attempt. */
export type FinanceRun = Schemas["FinanceRunInfo"];
/** A saved Statement password as listed: never its secret. */
export type PasswordItem = Schemas["PasswordItem"];
/** A page of Transactions. */
export type TransactionPage = Schemas["TransactionPage"];
/** A Transaction row. */
export type TransactionItem = Schemas["TransactionItem"];
/** One Statement's Printing of a Transaction, as that Statement printed it. */
export type PrintingItem = Schemas["PrintingItem"];
/** Query of `GET /api/finance/transactions`. */
export type TransactionQuery = NonNullable<paths["/api/finance/transactions"]["get"]["parameters"]["query"]>;
