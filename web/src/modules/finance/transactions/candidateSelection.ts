import type { TransactionItem, TransferCandidate } from "@/api/finance";

/** Chooses or drops a candidate. A Money-out row may be paid into several Money-in rows, so only it chooses several. */
export function toggleCandidate(selected: readonly number[], id: number, several: boolean): number[] {
  if (selected.includes(id)) return selected.filter((s) => s !== id);
  return several ? [...selected, id] : [id];
}

/** What the chosen candidates add up to, in paise. */
export function selectedPaise(candidates: readonly TransferCandidate[], selected: readonly number[]): number {
  return candidates.filter((c) => selected.includes(c.id)).reduce((sum, c) => sum + c.amount_paise, 0);
}

/** The body of `POST /transfers` for a row and the candidates chosen for it. */
export function linkRequest(row: Pick<TransactionItem, "id" | "direction">, selected: readonly number[]) {
  return row.direction === "debit"
    ? { debit_id: row.id, credit_ids: [...selected] }
    : { debit_id: selected[0] ?? 0, credit_ids: [row.id] };
}
