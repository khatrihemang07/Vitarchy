/** Query key of the Finance status. */
export const financeStatusKey = ["finance", "status"] as const;

/** Prefix of every Statements list key. */
export const statementsKeyPrefix = ["finance", "statements"] as const;

/** Query key of the Statements list for one status filter. */
export const statementsKey = (status: string) => ["finance", "statements", status] as const;

/** Query key of the Accounts list. */
export const accountsKey = ["finance", "accounts"] as const;

/** Query key of the saved Statement passwords. */
export const passwordsKey = ["finance", "passwords"] as const;

/** Query key of a Transactions page; the query is part of the key so each filter caches separately. */
export const transactionsKey = (query: unknown) => ["finance", "transactions", query] as const;

/** Prefix of every Transactions key, for invalidating them after a confirm or Read again. */
export const transactionsKeyPrefix = ["finance", "transactions"] as const;

/** Query key of one Transaction's Printings; under the Transactions prefix so a confirm or Read again refreshes it. */
export const printingsKey = (transactionId: number) => ["finance", "transactions", "printings", transactionId] as const;

/**
 * Query key of one Statement's preview. It sits under [`statementsKeyPrefix`], so every confirm
 * (which invalidates that prefix) refreshes each open preview and its Account matches.
 */
export const statementPreviewKey = (id: number) => [...statementsKeyPrefix, "preview", id] as const;

/** Query key of the Transactions a Transaction could be linked to; under the Transactions prefix so a link or an import refreshes it. */
export const transferCandidatesKey = (transactionId: number) => ["finance", "transactions", "transfer-candidates", transactionId] as const;
