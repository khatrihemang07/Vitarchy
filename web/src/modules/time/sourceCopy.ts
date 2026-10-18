import type { Source } from "@/api/time";

/** What the person reads about one Source's Import. */
export interface SourceCopy {
  /** The card title, and its button. */
  title: string;
  /** Where the Activity comes from and what Importing does with it. */
  description: string;
}

/** Wording per Source. A new Source is a new entry here, not a new branch. */
export const sourceCopy: Record<Source, SourceCopy> = {
  toggl: {
    title: "Import Toggl activity",
    description:
      "What the Toggl Track app noticed on this computer, kept as Activity. Importing never removes anything, so Vitarchy keeps what Toggl later discards.",
  },
  clockify: {
    title: "Import Clockify activity",
    description:
      "What the Clockify Desktop app noticed on this computer, with page addresses and idle time. Importing never removes anything, so Vitarchy keeps what Clockify later discards.",
  },
};
