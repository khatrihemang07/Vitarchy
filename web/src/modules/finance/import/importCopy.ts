import type { ConfirmCounts, SectionPreview } from "@/api/finance";
import { list, whenAny } from "../countPhrases";
import { formatCount } from "../money";

/** The import preview's count line, split so "new" can be emphasised: "3 new" and ", 1 already stored, 1 would gain detail". */
export function previewCountLine(section: Pick<SectionPreview, "new_count" | "already_stored_count" | "enriched_count">) {
  return {
    lead: `${formatCount(section.new_count)} new`,
    rest: `, ${list([`${formatCount(section.already_stored_count)} already stored`, whenAny(section.enriched_count, "would gain detail")])}`,
  };
}

/** What a confirm did, in plain words. */
export function confirmedLine(counts: ConfirmCounts): string {
  const parts = [
    `${formatCount(counts.added)} added`,
    `${formatCount(counts.already_stored)} already stored`,
    whenAny(counts.enriched, "gained detail"),
    whenAny(counts.linked, "Payment records linked"),
  ];
  return `${list(parts)}.`;
}
