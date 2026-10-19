/** Why the table is empty. */
export type EmptyReason = "filtered" | "empty-period" | "nothing-stored";

/** The reason, worked out in one place: filters on, else Transactions exist elsewhere, else none at all. */
export function emptyReason({ filtered, everStored }: { filtered: boolean; everStored: boolean }): EmptyReason {
  if (filtered) return "filtered";
  return everStored ? "empty-period" : "nothing-stored";
}

const COPY: Record<EmptyReason, { title: string; hint: string }> = {
  filtered: { title: "No Transactions match these filters", hint: "Try another Account, a different search or a wider Period." },
  "empty-period": { title: "No Transactions in this Period", hint: "Pick another Period to see the Transactions in it." },
  "nothing-stored": { title: "No Transactions yet", hint: "Confirm a Statement on the Import page and its Transactions appear here." },
};

/** What the empty table says for a reason. */
export function emptyCopy(reason: EmptyReason): { title: string; hint: string } {
  return COPY[reason];
}
