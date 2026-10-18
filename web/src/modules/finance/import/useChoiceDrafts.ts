import { useState } from "react";
import type { SectionPreview } from "@/api/finance";
import { type ChoiceDraft, initialDraft, isDraftComplete, toSectionChoice } from "../sectionChoice";

/**
 * The person's choice for every section of a Statement. A section the person has not touched
 * follows its preview, so a refreshed preview (another Statement was confirmed meanwhile)
 * moves it to the Account that now matches; an edited one keeps what was entered.
 */
export function useChoiceDrafts(sections: readonly SectionPreview[]) {
  const [edited, setEdited] = useState<Readonly<Record<number, ChoiceDraft>>>({});
  const drafts = sections.map((section, index) => edited[index] ?? initialDraft(section));
  return {
    drafts,
    setDraft: (index: number, next: ChoiceDraft) => {
      setEdited((all) => ({ ...all, [index]: next }));
    },
    /** Whether every section says enough to confirm. */
    ready: drafts.every(isDraftComplete),
    choices: () => drafts.map(toSectionChoice),
  };
}
