import { useState } from "react";
import type { SectionPreview } from "@/api/finance";
import { type ChoiceDraft, initialDraft, isDraftComplete, toSectionChoice } from "../sectionChoice";

/** The person's choice for every section of a Statement, starting from the matches and proposals. */
export function useChoiceDrafts(sections: readonly SectionPreview[]) {
  const [drafts, setDrafts] = useState<ChoiceDraft[]>(() => sections.map(initialDraft));
  return {
    drafts,
    setDraft: (index: number, next: ChoiceDraft) => {
      setDrafts((all) => all.map((d, i) => (i === index ? next : d)));
    },
    /** Whether every section says enough to confirm. */
    ready: drafts.length === sections.length && drafts.every(isDraftComplete),
    choices: () => drafts.map(toSectionChoice),
  };
}
