import type { ReadAgainCounts, ReadAllAgainCounts, SectionChanges } from "@/api/finance";
import { list, whenAny } from "../countPhrases";
import { formatCount } from "../money";

/** What a Read again of one section would do. */
export function sectionChangesLine(
  changes: Pick<SectionChanges, "add_count" | "new_printing_count" | "enriched_count" | "unchanged_count" | "no_longer_found_count">,
): string {
  const parts = [
    `${formatCount(changes.add_count)} to add`,
    whenAny(changes.new_printing_count, "Printings to add to stored Transactions"),
    whenAny(changes.enriched_count, "would gain detail"),
    `${formatCount(changes.unchanged_count)} unchanged`,
    `${formatCount(changes.no_longer_found_count)} no longer found (marked, never deleted)`,
  ];
  return `${list(parts)}.`;
}

/** What an applied Read again did; the Read all again counts add Statements and failures separately. */
export function appliedLine(counts: ReadAgainCounts | ReadAllAgainCounts): string {
  const unchanged = "unchanged" in counts ? `${formatCount(counts.unchanged)} unchanged` : null;
  const parts = [
    `${formatCount(counts.added)} added`,
    whenAny(counts.printings_added, "Printings added to stored Transactions"),
    whenAny(counts.enriched, "gained detail"),
    unchanged,
    `${formatCount(counts.no_longer_found)} no longer found`,
    whenAny(counts.linked, "Payment records linked"),
  ];
  return list(parts);
}
