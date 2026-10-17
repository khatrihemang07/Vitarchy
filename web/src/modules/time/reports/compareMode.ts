import type { PeriodKind, Report, ReportGroup } from "@/api/time";
import { USUAL_LABEL, USUAL_PHRASE } from "../usualWords";
import { dayCount, periodLabel } from "./period";
import type { ReportSearch } from "./reportSearch";

/** What a Report compares with: the Previous Period, or the Usual (the last 4 weeks). */
export type CompareMode = "previous" | "usual";

type GroupChange = NonNullable<ReportGroup["vs_previous"]["change"]>;
type OverviewChange = Report["overview"]["vs_previous"];

/** How one group compares with the chosen reference. */
export interface GroupComparison {
  change: GroupChange | null;
  /** True when the group has no time in the reference. */
  isNew: boolean;
  /** The Usual's own figures; null unless comparing with the Usual and the group happened then. */
  usual: ReportGroup["vs_usual"]["usual"] | null;
}

/** Everything that differs between the modes, kept together: its words, and how it reads a Report. */
interface CompareDescriptor {
  /** The toggle's label. */
  label: string;
  /** How a sentence names it. */
  phrase: string;
  /** A group's comparison. */
  group: (group: ReportGroup) => GroupComparison;
  /** The overview's changes. */
  overview: (overview: Report["overview"]) => OverviewChange;
  /**
   * The change of a one-Day total. Against the Usual it is the Usual's average per Day (a
   * Day is one Day, not a scaled total); against the Previous Period, yesterday's total.
   */
  dayTotalChange: (change: GroupChange) => number;
  /** The sentence under the overview saying what the changes are measured against. */
  note: (report: Report) => string;
}

function daysWithEntries(n: number): string {
  if (n === 0) return "no Days with entries";
  return `${String(n)} ${n === 1 ? "Day" : "Days"} with entries`;
}

/** What limits the Usual: nothing, or how many Days of the person's history it can use. */
function historyHint(available: number, range: number): string {
  if (available >= range) return "";
  if (available === 0) return ". No history yet";
  return `. Only ${String(available)} ${available === 1 ? "Day" : "Days"} of history so far`;
}

/** The modes, in toggle order. */
export const COMPARE: Record<CompareMode, CompareDescriptor> = {
  previous: {
    label: "Previous Period",
    phrase: "the Previous Period",
    group: ({ vs_previous: { change, is_new } }) => ({ change: change ?? null, isNew: is_new, usual: null }),
    overview: (overview) => overview.vs_previous,
    dayTotalChange: (change) => change.total_s,
    note: ({ period }) => `Compared with the Previous Period, ${periodLabel(period.kind, { from: period.previous_from, to: period.previous_to })}`,
  },
  usual: {
    label: USUAL_LABEL,
    phrase: USUAL_PHRASE,
    group: ({ vs_usual: { change, is_new, usual } }) => ({ change: change ?? null, isNew: is_new, usual: usual ?? null }),
    overview: (overview) => overview.vs_usual,
    dayTotalChange: (change) => change.per_day.avg_s,
    note: ({ period, overview }) => {
      const range = { from: period.usual_from, to: period.usual_to };
      const dates = periodLabel("custom", range);
      return `Compared with ${USUAL_PHRASE}, ${dates} (${daysWithEntries(overview.usual_days)})${historyHint(period.usual_available_days, dayCount(range))}`;
    },
  },
};

/** Every mode, in toggle order. */
export const COMPARE_MODES = Object.keys(COMPARE) as CompareMode[];

/** The sentence under the overview for the chosen mode. */
export function compareNote(report: Report, compare: CompareMode): string {
  return COMPARE[compare].note(report);
}

/** A single Day says little against yesterday, so it opens on the Usual; longer Periods on the Previous Period. */
export function defaultCompare(kind: PeriodKind): CompareMode {
  return kind === "day" ? "usual" : "previous";
}

/** The mode in effect: the person's choice from the URL, else the default for the Period kind. */
export function compareOf(search: Pick<ReportSearch, "kind" | "compare">): CompareMode {
  return search.compare ?? defaultCompare(search.kind);
}
