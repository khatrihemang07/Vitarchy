import { failureFromCode, type ReadAgainItem } from "@/api/finance";
import { failureMessage } from "../failureMessage";

/** What one Statement's Read again would change, summed over its sections or records. */
export interface ItemChanges {
  statementId: number;
  fileName: string;
  add: number;
  unchanged: number;
  noLongerFound: number;
}

/** The combined preview of Read all again, derived from the server's items. */
export interface ReadAllAgainSummary {
  /** Statements whose Read again adds rows or marks rows no longer found. */
  changed: ItemChanges[];
  /** Statements that could not be read again, with words for why. */
  failed: { statementId: number; fileName: string; reason: string }[];
  add: number;
  unchanged: number;
  noLongerFound: number;
}

function changesOf(item: ReadAgainItem): ItemChanges {
  const preview = item.preview;
  const totals = { add: 0, unchanged: 0, noLongerFound: 0 };
  for (const section of preview?.sections ?? []) {
    totals.add += section.add_count;
    totals.unchanged += section.unchanged_count;
    totals.noLongerFound += section.no_longer_found_count;
  }
  if (preview?.payment_records) {
    totals.add += preview.payment_records.add_count;
    totals.unchanged += preview.payment_records.unchanged_count;
  }
  return { statementId: item.statement_id, fileName: item.file_name, ...totals };
}

function reasonFor(code: string): string {
  const known = failureFromCode(code);
  if (known) return failureMessage(known);
  return code.startsWith("reader_") ? failureMessage({ kind: "reader_failed" }) : "It could not be read again.";
}

/** Totals, the Statements with changes and the ones that failed, from the per-Statement items. */
export function summariseReadAllAgain(items: ReadAgainItem[]): ReadAllAgainSummary {
  const summary: ReadAllAgainSummary = { changed: [], failed: [], add: 0, unchanged: 0, noLongerFound: 0 };
  for (const item of items) {
    if (item.error_code) {
      summary.failed.push({ statementId: item.statement_id, fileName: item.file_name, reason: reasonFor(item.error_code) });
      continue;
    }
    const changes = changesOf(item);
    summary.add += changes.add;
    summary.unchanged += changes.unchanged;
    summary.noLongerFound += changes.noLongerFound;
    if (changes.add > 0 || changes.noLongerFound > 0) summary.changed.push(changes);
  }
  return summary;
}
