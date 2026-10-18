import { failureFromCode, type ReadAgainItem } from "@/api/finance";
import { failureMessage } from "../failureMessage";
import { formatCount } from "../money";

/** What one Statement's Read again would change, summed over its sections or records. */
export interface ItemChanges {
  statementId: number;
  fileName: string;
  add: number;
  /** Printings that would be added to Transactions other Statements stored. */
  newPrintings: number;
  /** Of the rows to add or match, those that would gain detail. */
  enriched: number;
  unchanged: number;
  noLongerFound: number;
}

/** The combined preview of Read all again, derived from the server's items. */
export interface ReadAllAgainSummary {
  /** Statements whose Read again adds rows or Printings, or marks Printings no longer found. */
  changed: ItemChanges[];
  /** Statements that could not be read again, with words for why. */
  failed: { statementId: number; fileName: string; reason: string }[];
  add: number;
  newPrintings: number;
  enriched: number;
  unchanged: number;
  noLongerFound: number;
}

function changesOf(item: ReadAgainItem): ItemChanges {
  const preview = item.preview;
  const totals = { add: 0, newPrintings: 0, enriched: 0, unchanged: 0, noLongerFound: 0 };
  for (const section of preview?.sections ?? []) {
    totals.add += section.add_count;
    totals.newPrintings += section.new_printing_count;
    totals.enriched += section.enriched_count;
    totals.unchanged += section.unchanged_count;
    totals.noLongerFound += section.no_longer_found_count;
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
  const summary: ReadAllAgainSummary = { changed: [], failed: [], add: 0, newPrintings: 0, enriched: 0, unchanged: 0, noLongerFound: 0 };
  for (const item of items) {
    if (item.error_code) {
      summary.failed.push({ statementId: item.statement_id, fileName: item.file_name, reason: reasonFor(item.error_code) });
      continue;
    }
    const changes = changesOf(item);
    summary.add += changes.add;
    summary.newPrintings += changes.newPrintings;
    summary.enriched += changes.enriched;
    summary.unchanged += changes.unchanged;
    summary.noLongerFound += changes.noLongerFound;
    if (changes.add > 0 || changes.newPrintings > 0 || changes.noLongerFound > 0) summary.changed.push(changes);
  }
  return summary;
}

/** One Statement's line in the combined preview: what its Read again would add or mark. */
export function itemChangesLine(c: ItemChanges): string {
  const parts = [`${formatCount(c.add)} to add`];
  if (c.newPrintings > 0) parts.push(`${formatCount(c.newPrintings)} Printings to add`);
  if (c.enriched > 0) parts.push(`${formatCount(c.enriched)} would gain detail`);
  parts.push(`${formatCount(c.noLongerFound)} no longer found`);
  return parts.join(", ");
}
